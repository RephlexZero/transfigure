use std::sync::Arc;

use leptos::prelude::*;
use web_sys::js_sys;

use crate::types::{BatchFile, FileStatus, FileView, Phase};
use crate::utils::{download_blob, download_blob_raw, make_output_name, next_tick};

/// Archive formats for saving a batch: (value, label).
pub const ARCHIVE_FORMATS: [(&str, &str); 4] = [
    ("zip", ".zip"),
    ("tar.gz", ".tar.gz"),
    ("tar.xz", ".tar.xz"),
    ("7z", ".7z"),
];

/// Output formats that take a lossy quality setting (for MP3 it sets the
/// bitrate).
fn is_lossy_target(fmt: &str) -> bool {
    matches!(fmt, "jpg" | "jpeg" | "avif" | "mp3")
}

/// Totals for the whole batch, recomputed once per change.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Stats {
    pub count: usize,
    pub total_size: usize,
    /// Files with a target that haven't been converted yet.
    pub ready: usize,
    pub done: usize,
    pub failed: usize,
    pub output_size: usize,
    /// A queued file targets a lossy format, so the quality control matters.
    pub lossy_queued: bool,
}

/// The file batch and everything done to it. Copyable handle shared through
/// context, so the specimen, queue and format grid all see one state.
#[derive(Clone, Copy)]
pub struct Batch {
    files: RwSignal<Vec<BatchFile>>,
    next_id: StoredValue<usize>,
    focused: RwSignal<Option<usize>>,
    pub converting: RwSignal<bool>,
    /// (finished, total) for the run in progress.
    pub progress: RwSignal<(usize, usize)>,
    pub quality: RwSignal<u8>,
    pub archive: RwSignal<&'static str>,
    pub views: Memo<Vec<FileView>>,
    /// The file the specimen shows: the focused one, else the first.
    pub shown: Memo<Option<FileView>>,
    pub stats: Memo<Stats>,
    /// Output formats every file supports, for "set all to". Empty for a
    /// single file, where it would duplicate the file's own picker.
    pub common_formats: Memo<Vec<&'static str>>,
    /// The target every file shares, if they all agree.
    pub uniform_target: Memo<Option<String>>,
}

impl Batch {
    pub fn new() -> Self {
        let files = RwSignal::new(Vec::<BatchFile>::new());
        let focused = RwSignal::new(None::<usize>);

        let views = Memo::new(move |_| {
            files.with(|list| {
                list.iter()
                    .enumerate()
                    .map(|(i, f)| FileView::from_file(i + 1, f))
                    .collect::<Vec<_>>()
            })
        });
        let shown = Memo::new(move |_| {
            views.with(|v| {
                focused
                    .get()
                    .and_then(|id| v.iter().find(|f| f.id == id))
                    .or_else(|| v.first())
                    .cloned()
            })
        });
        let stats = Memo::new(move |_| {
            views.with(|v| {
                let mut s = Stats {
                    count: v.len(),
                    ..Stats::default()
                };
                for f in v {
                    s.total_size += f.size;
                    match &f.phase {
                        Phase::Pending if f.target.is_some() => {
                            s.ready += 1;
                            if f.target.as_deref().is_some_and(is_lossy_target) {
                                s.lossy_queued = true;
                            }
                        }
                        Phase::Done { out_size, .. } => {
                            s.done += 1;
                            s.output_size += out_size;
                        }
                        Phase::Failed(_) => s.failed += 1,
                        _ => {}
                    }
                }
                s
            })
        });
        let common_formats = Memo::new(move |_| {
            views.with(|v| {
                if v.len() < 2 {
                    return Vec::new();
                }
                let mut common = v[0].formats.clone();
                for f in &v[1..] {
                    common.retain(|c| f.formats.contains(c));
                }
                common
            })
        });
        let uniform_target = Memo::new(move |_| {
            views.with(|v| {
                let first = v.first()?.target.clone()?;
                v.iter()
                    .all(|f| f.target.as_deref() == Some(first.as_str()))
                    .then_some(first)
            })
        });

        Self {
            files,
            next_id: StoredValue::new(0),
            focused,
            converting: RwSignal::new(false),
            progress: RwSignal::new((0, 0)),
            quality: RwSignal::new(85),
            archive: RwSignal::new("zip"),
            views,
            shown,
            stats,
            common_formats,
            uniform_target,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stats.with(|s| s.count == 0)
    }

    /// Ids in queue order, for keyed lists.
    pub fn ids(&self) -> Vec<usize> {
        self.views.with(|v| v.iter().map(|f| f.id).collect())
    }

    pub fn view_of(&self, id: usize) -> Option<FileView> {
        self.views.with(|v| v.iter().find(|f| f.id == id).cloned())
    }

    pub fn shown_id(&self) -> Option<usize> {
        self.shown.with(|s| s.as_ref().map(|f| f.id))
    }

    /// Canonical extensions currently in the queue.
    pub fn has_ext(&self, canonical: &str) -> bool {
        self.views
            .with(|v| v.iter().any(|f| converter::canonical(&f.ext) == canonical))
    }

    pub fn focus(&self, id: usize) {
        self.focused.set(Some(id));
    }

    /// Queue new files, each defaulting to its most useful output, and
    /// show the first of them.
    pub fn add(&self, new_files: Vec<(String, Vec<u8>)>) {
        let mut first_new = None;
        self.files.update(|list| {
            for (name, bytes) in new_files {
                let extension = converter::detect_format(&name).unwrap_or_default();
                let formats = converter::get_output_formats(&extension);
                let id = self.next_id.get_value();
                self.next_id.set_value(id + 1);
                first_new.get_or_insert(id);
                list.push(BatchFile {
                    id,
                    size: bytes.len(),
                    bytes: Arc::from(bytes),
                    target: formats.first().map(|s| s.to_string()),
                    formats,
                    extension,
                    name,
                    status: FileStatus::Pending,
                });
            }
        });
        if first_new.is_some() {
            self.focused.set(first_new);
        }
    }

    pub fn remove(&self, id: usize) {
        self.files.update(|list| list.retain(|f| f.id != id));
        if self.focused.get_untracked() == Some(id) {
            self.focused.set(None);
        }
    }

    pub fn clear(&self) {
        self.files.set(Vec::new());
        self.focused.set(None);
        self.converting.set(false);
    }

    pub fn set_target(&self, id: usize, target: String) {
        self.files.update(|list| {
            if let Some(f) = list.iter_mut().find(|f| f.id == id) {
                f.target = Some(target);
                // A finished file goes back to pending so it can be redone.
                if f.status.is_finished() {
                    f.status = FileStatus::Pending;
                }
            }
        });
    }

    pub fn set_all_targets(&self, target: String) {
        self.files.update(|list| {
            for f in list.iter_mut() {
                if f.formats.contains(&target.as_str()) {
                    f.target = Some(target.clone());
                    if f.status.is_finished() {
                        f.status = FileStatus::Pending;
                    }
                }
            }
        });
    }

    /// Convert queued files one at a time, yielding to the browser between
    /// steps so each status change actually paints. Conversion itself is
    /// synchronous CPU work on the main thread; without a macrotask yield the
    /// whole batch would block rendering until it finished. The specimen
    /// follows whichever file is converting.
    pub fn convert_all(&self) {
        if self.converting.get_untracked() {
            return;
        }
        self.converting.set(true);
        let batch = *self;
        let quality = self.quality.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            let queue: Vec<usize> = batch.files.with_untracked(|list| {
                list.iter()
                    .filter(|f| f.target.is_some() && f.status == FileStatus::Pending)
                    .map(|f| f.id)
                    .collect()
            });
            batch.progress.set((0, queue.len()));

            for (n, id) in queue.into_iter().enumerate() {
                let Some((bytes, from, to)) = batch.files.with_untracked(|list| {
                    list.iter().find(|f| f.id == id).and_then(|f| {
                        Some((f.bytes.clone(), f.extension.clone(), f.target.clone()?))
                    })
                }) else {
                    continue; // removed while the batch was running
                };

                batch.set_status(id, FileStatus::Converting);
                batch.focused.set(Some(id));
                next_tick().await;

                let config =
                    serde_json::json!({ "from": from, "to": to, "quality": quality }).to_string();
                let started = js_sys::Date::now();
                let result = converter::convert(&bytes, &config);
                let elapsed_ms = (js_sys::Date::now() - started).max(0.0) as u32;

                batch.set_status(
                    id,
                    match result {
                        Ok(data) => FileStatus::Done {
                            data: Arc::from(data),
                            elapsed_ms,
                        },
                        Err(e) => FileStatus::Error(e),
                    },
                );
                batch.progress.update(|p| p.0 = n + 1);
                next_tick().await;
            }

            batch.converting.set(false);
        });
    }

    fn set_status(&self, id: usize, status: FileStatus) {
        self.files.update(|list| {
            if let Some(f) = list.iter_mut().find(|f| f.id == id) {
                f.status = status;
            }
        });
    }

    pub fn save(&self, id: usize) {
        self.files.with_untracked(|list| {
            if let Some(f) = list.iter().find(|f| f.id == id)
                && let FileStatus::Done { ref data, .. } = f.status
            {
                download_blob(data, &f.name, f.target.as_deref().unwrap_or("bin"));
            }
        });
    }

    /// The only finished file, when exactly one is done.
    pub fn single_done(&self) -> Option<FileView> {
        self.views.with(|v| {
            let mut done = v.iter().filter(|f| matches!(f.phase, Phase::Done { .. }));
            let first = done.next()?;
            done.next().is_none().then(|| first.clone())
        })
    }

    pub fn save_all(&self) {
        let entries: Vec<converter::archive::ArchiveEntry> = self.files.with_untracked(|list| {
            list.iter()
                .filter_map(|f| match &f.status {
                    FileStatus::Done { data, .. } => Some(converter::archive::ArchiveEntry {
                        name: make_output_name(&f.name, f.target.as_deref().unwrap_or("bin")),
                        data: data.to_vec(),
                    }),
                    _ => None,
                })
                .collect()
        });
        if entries.is_empty() {
            return;
        }

        let archive = match self.archive.get_untracked() {
            "zip" => {
                converter::archive::create_zip(&entries).map(|d| (d, "zip", "application/zip"))
            }
            "tar.gz" => converter::archive::create_tar_gz(&entries)
                .map(|d| (d, "tar.gz", "application/gzip")),
            "tar.xz" => converter::archive::create_tar_xz(&entries)
                .map(|d| (d, "tar.xz", "application/x-xz")),
            "7z" => converter::archive::create_7z(&entries)
                .map(|d| (d, "7z", "application/x-7z-compressed")),
            _ => return,
        };

        match archive {
            Ok((data, ext, mime)) => {
                download_blob_raw(&data, &format!("transfigure-output.{ext}"), mime)
            }
            Err(e) => web_sys::console::error_1(&format!("Archive error: {e}").into()),
        }
    }
}

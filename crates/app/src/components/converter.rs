use std::sync::Arc;

use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

use super::icons::{CategoryTile, Ic, Icon};
use crate::types::{BatchFile, FileStatus};
use crate::utils::{
    category_of, download_blob, download_blob_raw, format_elapsed, format_size, format_size_delta,
    make_output_name, next_tick,
};

/// Output formats that take a lossy quality setting.
fn is_lossy_target(fmt: &str) -> bool {
    matches!(fmt, "jpg" | "jpeg" | "avif")
}

const ARCHIVE_FORMATS: [(&str, &str); 4] = [
    ("zip", "ZIP"),
    ("tar.gz", "TAR.GZ"),
    ("tar.xz", "TAR.XZ"),
    ("7z", "7Z"),
];

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Read every file in a `FileList` and hand them over together once all
/// have loaded, so a dropped batch lands in one update.
fn read_files(
    file_list: web_sys::FileList,
    add: impl Fn(Vec<(String, Vec<u8>)>) + Clone + 'static,
) {
    let count = file_list.length();
    if count == 0 {
        return;
    }
    let collected = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let remaining = std::rc::Rc::new(std::cell::Cell::new(count));

    for i in 0..count {
        let Some(file) = file_list.get(i) else {
            remaining.set(remaining.get() - 1);
            continue;
        };
        let collected = collected.clone();
        let remaining = remaining.clone();
        let add = add.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let name = file.name();
            if let Ok(buf) = JsFuture::from(file.array_buffer()).await {
                let array = js_sys::Uint8Array::new(&buf);
                collected.borrow_mut().push((name, array.to_vec()));
            }
            let left = remaining.get() - 1;
            remaining.set(left);
            if left == 0 {
                let items = collected.borrow_mut().drain(..).collect();
                add(items);
            }
        });
    }
}

fn drag_has_files(ev: &web_sys::DragEvent) -> bool {
    ev.data_transfer()
        .is_some_and(|dt| dt.types().includes(&"Files".into(), 0))
}

fn is_mac() -> bool {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .is_some_and(|ua| ua.contains("Mac"))
}

#[component]
pub fn ConverterSection(
    files: RwSignal<Vec<BatchFile>>,
    next_id: RwSignal<usize>,
) -> impl IntoView {
    let is_converting = RwSignal::new(false);
    let quality = RwSignal::new(85u8);
    let archive_format = RwSignal::new("zip");
    // (finished, total) for the batch currently converting.
    let progress = RwSignal::new((0usize, 0usize));
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let has_files = Memo::new(move |_| files.with(|f| !f.is_empty()));
    let file_count = Memo::new(move |_| files.with(|f| f.len()));
    let total_size = Memo::new(move |_| files.with(|f| f.iter().map(|f| f.size).sum::<usize>()));
    let all_done = Memo::new(move |_| {
        files.with(|f| !f.is_empty() && f.iter().all(|f| f.status.is_finished()))
    });
    let ready_count = Memo::new(move |_| {
        files.with(|f| {
            f.iter()
                .filter(|f| f.target.is_some() && f.status == FileStatus::Pending)
                .count()
        })
    });
    let can_convert = Memo::new(move |_| ready_count.get() > 0 && !is_converting.get());
    let done_count = Memo::new(move |_| {
        files.with(|f| {
            f.iter()
                .filter(|f| matches!(f.status, FileStatus::Done { .. }))
                .count()
        })
    });
    let error_count = Memo::new(move |_| {
        files.with(|f| {
            f.iter()
                .filter(|f| matches!(f.status, FileStatus::Error(_)))
                .count()
        })
    });
    let output_size = Memo::new(move |_| {
        files.with(|f| {
            f.iter()
                .map(|f| match &f.status {
                    FileStatus::Done { data, .. } => data.len(),
                    _ => 0,
                })
                .sum::<usize>()
        })
    });
    // Show the quality control when any queued file targets a lossy format.
    let show_quality = Memo::new(move |_| {
        files.with(|f| {
            f.iter().any(|f| {
                f.status == FileStatus::Pending && f.target.as_deref().is_some_and(is_lossy_target)
            })
        })
    });
    // Output formats every file in the batch supports — drives "convert all to".
    let common_formats = Memo::new(move |_| {
        files.with(|f| {
            if f.len() < 2 {
                return Vec::new();
            }
            let mut common: Vec<&'static str> = converter::get_output_formats(&f[0].extension);
            for file in &f[1..] {
                let fmts = converter::get_output_formats(&file.extension);
                common.retain(|c| fmts.contains(c));
            }
            common
        })
    });
    // The target shared by every file, if they all agree.
    let uniform_target = Memo::new(move |_| {
        files.with(|f| {
            let first = f.first()?.target.clone()?;
            f.iter()
                .all(|x| x.target.as_deref() == Some(first.as_str()))
                .then_some(first)
        })
    });

    let add_files = move |new_files: Vec<(String, Vec<u8>)>| {
        files.update(|list| {
            for (name, bytes) in new_files {
                let size = bytes.len();
                let ext = converter::detect_format(&name).unwrap_or_default();
                let formats = converter::get_output_formats(&ext);
                let default_target = formats.first().map(|s| s.to_string());
                let id = next_id.get_untracked();
                next_id.set(id + 1);
                list.push(BatchFile {
                    id,
                    name,
                    bytes: Arc::from(bytes),
                    extension: ext,
                    size,
                    target: default_target,
                    status: FileStatus::Pending,
                });
            }
        });
    };

    let open_picker = move || {
        if let Some(input) = input_ref.get_untracked() {
            let el: &web_sys::HtmlInputElement = &input;
            el.click();
        }
    };

    let on_input_change = move |_ev: ev::Event| {
        if let Some(input) = input_ref.get_untracked() {
            let el: &web_sys::HtmlInputElement = &input;
            if let Some(list) = el.files() {
                read_files(list, add_files);
            }
            el.set_value("");
        }
    };

    // ── Page-wide drop target and paste ─────────────
    // Counter rather than bool: dragenter/dragleave fire for every element
    // crossed, and a bool flickers off mid-drag.
    let drag_depth = RwSignal::new(0i32);
    let dragging = Memo::new(move |_| drag_depth.get() > 0);

    let _ = window_event_listener(ev::dragenter, move |ev| {
        if drag_has_files(&ev) {
            ev.prevent_default();
            drag_depth.update(|d| *d += 1);
        }
    });
    let _ = window_event_listener(ev::dragover, move |ev| {
        if drag_has_files(&ev) {
            ev.prevent_default();
        }
    });
    let _ = window_event_listener(ev::dragleave, move |ev| {
        if drag_has_files(&ev) {
            drag_depth.update(|d| *d = (*d - 1).max(0));
        }
    });
    let _ = window_event_listener(ev::drop, move |ev| {
        drag_depth.set(0);
        if let Some(list) = ev.data_transfer().and_then(|dt| dt.files())
            && list.length() > 0
        {
            ev.prevent_default();
            read_files(list, add_files);
        }
    });
    let _ = window_event_listener_untyped("paste", move |ev| {
        let ev: web_sys::ClipboardEvent = ev.unchecked_into();
        if let Some(list) = ev.clipboard_data().and_then(|dt| dt.files())
            && list.length() > 0
        {
            ev.prevent_default();
            read_files(list, add_files);
        }
    });

    // ── Batch actions ───────────────────────────────

    let remove_file = move |file_id: usize| {
        files.update(|list| list.retain(|f| f.id != file_id));
    };

    let set_target = move |file_id: usize, target: String| {
        files.update(|list| {
            if let Some(f) = list.iter_mut().find(|f| f.id == file_id) {
                f.target = Some(target);
                // Allow re-converting: a finished file goes back to Pending
                // so the Convert button re-appears.
                if f.status.is_finished() {
                    f.status = FileStatus::Pending;
                }
            }
        });
    };

    let set_all_targets = move |target: String| {
        files.update(|list| {
            for f in list.iter_mut() {
                if converter::get_output_formats(&f.extension).contains(&target.as_str()) {
                    f.target = Some(target.clone());
                    if f.status.is_finished() {
                        f.status = FileStatus::Pending;
                    }
                }
            }
        });
    };

    // Convert queued files one at a time, yielding to the browser between
    // steps so each status change actually paints. Conversion itself is
    // synchronous CPU work on the main thread; without a macrotask yield the
    // whole batch would block rendering until it finished.
    let convert_all = move |_| {
        if is_converting.get_untracked() {
            return;
        }
        is_converting.set(true);
        let q = quality.get_untracked();

        wasm_bindgen_futures::spawn_local(async move {
            let queue: Vec<usize> = files.with_untracked(|list| {
                list.iter()
                    .filter(|f| f.target.is_some() && f.status == FileStatus::Pending)
                    .map(|f| f.id)
                    .collect()
            });
            progress.set((0, queue.len()));

            for (n, file_id) in queue.into_iter().enumerate() {
                let Some((bytes, ext, target_ext)) = files.with_untracked(|list| {
                    list.iter()
                        .find(|f| f.id == file_id)
                        .map(|f| (f.bytes.clone(), f.extension.clone(), f.target.clone()))
                }) else {
                    continue; // removed while the batch was running
                };
                let Some(target_ext) = target_ext else {
                    continue;
                };

                files.update(|list| {
                    if let Some(f) = list.iter_mut().find(|f| f.id == file_id) {
                        f.status = FileStatus::Converting;
                    }
                });
                next_tick().await;

                let config = serde_json::json!({
                    "from": ext,
                    "to": target_ext,
                    "quality": q,
                })
                .to_string();

                let started = js_sys::Date::now();
                let result = converter::convert(&bytes, &config);
                let elapsed_ms = (js_sys::Date::now() - started).max(0.0) as u32;

                files.update(|list| {
                    if let Some(f) = list.iter_mut().find(|f| f.id == file_id) {
                        f.status = match result {
                            Ok(data) => FileStatus::Done {
                                data: Arc::from(data),
                                elapsed_ms,
                            },
                            Err(e) => FileStatus::Error(e),
                        };
                    }
                });
                progress.update(|p| p.0 = n + 1);
                next_tick().await;
            }

            is_converting.set(false);
        });
    };

    let on_reset = move |_| {
        files.set(Vec::new());
        is_converting.set(false);
    };

    let save_file = move |file_id: usize| {
        files.with_untracked(|list| {
            if let Some(f) = list.iter().find(|f| f.id == file_id)
                && let FileStatus::Done { ref data, .. } = f.status
            {
                let target_ext = f.target.as_deref().unwrap_or("bin");
                download_blob(data, &f.name, target_ext);
            }
        });
    };

    let save_first_done = move || {
        let id = files.with_untracked(|list| {
            list.iter()
                .find(|f| matches!(f.status, FileStatus::Done { .. }))
                .map(|f| f.id)
        });
        if let Some(id) = id {
            save_file(id);
        }
    };

    let save_all = move || {
        let entries: Vec<converter::archive::ArchiveEntry> = files.with_untracked(|list| {
            list.iter()
                .filter_map(|f| {
                    if let FileStatus::Done { ref data, .. } = f.status {
                        let target_ext = f.target.as_deref().unwrap_or("bin");
                        Some(converter::archive::ArchiveEntry {
                            name: make_output_name(&f.name, target_ext),
                            data: data.to_vec(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        });

        if entries.is_empty() {
            return;
        }

        let archive_result = match archive_format.get_untracked() {
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

        match archive_result {
            Ok((archive_data, archive_ext, mime)) => download_blob_raw(
                &archive_data,
                &format!("transfigure-output.{archive_ext}"),
                mime,
            ),
            Err(e) => web_sys::console::error_1(&format!("Archive error: {e}").into()),
        }
    };

    let accept_str: String = converter::ALL_INPUT_FORMATS
        .iter()
        .map(|ext| format!(".{ext}"))
        .collect::<Vec<_>>()
        .join(",");

    view! {
        <input
            node_ref=input_ref
            type="file"
            multiple=true
            accept=accept_str
            class="hidden"
            tabindex="-1"
            aria-hidden="true"
            on:change=on_input_change
        />

        <DragOverlay dragging=dragging/>

        <div id="converter" class="card overflow-hidden scroll-mt-24">
            {move || {
                if !has_files.get() {
                    return view! { <EmptyState open_picker=open_picker/> }.into_any();
                }
                view! {
                    // Toolbar
                    <div class="flex items-center justify-between gap-3 pl-4 pr-2 sm:pl-5 sm:pr-3 h-14 border-b border-line">
                        <p class="flex items-baseline gap-2 min-w-0">
                            <span class="text-sm font-semibold">{move || plural(file_count.get(), "file", "files")}</span>
                            <span class="text-xs text-subtle tabular-nums">{move || format_size(total_size.get())}</span>
                        </p>
                        <div class="flex items-center gap-0.5">
                            <button type="button" class="btn btn-sm btn-ghost" on:click=move |_| open_picker()>
                                <Icon icon=Ic::Plus class="size-4"/>
                                "Add"
                                <span class="hidden sm:inline -ml-1">" files"</span>
                            </button>
                            <button
                                type="button"
                                class="btn btn-sm btn-ghost hover:text-bad"
                                prop:disabled=move || is_converting.get()
                                on:click=on_reset
                            >
                                "Clear"
                            </button>
                        </div>
                    </div>

                    <OptionsBar
                        common_formats=common_formats
                        uniform_target=uniform_target
                        set_all_targets=set_all_targets
                        quality=quality
                        show_quality=show_quality
                        is_converting=is_converting
                    />

                    <ul class="divide-y divide-line max-h-[min(28rem,60vh)] overflow-y-auto overscroll-contain" aria-label="Files">
                        <For
                            each=move || files.with(|l| l.iter().map(|f| f.id).collect::<Vec<_>>())
                            key=|id| *id
                            let:id
                        >
                            <FileRow
                                id=id
                                files=files
                                on_remove=remove_file
                                on_set_target=set_target
                                on_save=save_file
                            />
                        </For>
                    </ul>

                    // Action bar
                    <div class="relative flex flex-col-reverse sm:flex-row sm:items-center justify-between gap-3 px-4 sm:px-5 py-3.5 border-t border-line bg-sunken/50">
                        {move || is_converting.get().then(|| {
                            let pct = move || {
                                let (d, t) = progress.get();
                                if t == 0 { 0.0 } else { d as f64 / t as f64 * 100.0 }
                            };
                            view! {
                                <div class="absolute inset-x-0 top-0 h-0.5 bg-accent/15" aria-hidden="true">
                                    <div class="h-full bg-accent transition-[width] duration-300 ease-out" style:width=move || format!("{}%", pct())></div>
                                </div>
                            }
                        })}

                        <p class="text-[13px] text-muted tabular-nums" role="status" aria-live="polite">
                            {move || {
                                if is_converting.get() {
                                    let (d, t) = progress.get();
                                    format!("Converting {} of {t}…", (d + 1).min(t))
                                } else if all_done.get() {
                                    let ok = done_count.get();
                                    let err = error_count.get();
                                    let mut s = format!("{} converted", plural(ok, "file", "files"));
                                    if ok > 0 {
                                        s.push_str(&format!(" · {}", format_size(output_size.get())));
                                    }
                                    if err > 0 {
                                        s.push_str(&format!(" · {err} failed"));
                                    }
                                    s
                                } else {
                                    format!("{} ready to convert", plural(ready_count.get(), "file", "files"))
                                }
                            }}
                        </p>

                        <div class="flex items-center gap-2 sm:justify-end">
                            {move || {
                                if all_done.get() {
                                    let done = done_count.get();
                                    view! {
                                        <button type="button" class="btn btn-ghost px-2.5 sm:px-3.5" on:click=on_reset aria-label="Start over" title="Start over">
                                            <Icon icon=Ic::RotateCcw class="size-4"/>
                                            <span class="hidden sm:inline">"Start over"</span>
                                        </button>
                                        {(done >= 2).then(|| view! {
                                            <select
                                                class="select h-9 ml-auto sm:ml-0"
                                                aria-label="Archive format"
                                                on:change=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    if let Some((fmt, _)) = ARCHIVE_FORMATS.iter().find(|(f, _)| *f == v) {
                                                        archive_format.set(fmt);
                                                    }
                                                }
                                            >
                                                {ARCHIVE_FORMATS.iter().map(|(fmt, label)| view! {
                                                    <option value=*fmt prop:selected=move || archive_format.get() == *fmt>{*label}</option>
                                                }).collect::<Vec<_>>()}
                                            </select>
                                        })}
                                        {(done > 0).then(|| view! {
                                            <button
                                                type="button"
                                                class="btn btn-primary flex-1 sm:flex-none"
                                                on:click=move |_| { if done >= 2 { save_all() } else { save_first_done() } }
                                            >
                                                <Icon icon=Ic::Download class="size-4"/>
                                                {if done >= 2 { "Download all" } else { "Download" }}
                                            </button>
                                        })}
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            type="button"
                                            class="btn btn-primary w-full sm:w-auto sm:min-w-[11rem]"
                                            prop:disabled=move || !can_convert.get()
                                            on:click=convert_all
                                        >
                                            {move || if is_converting.get() {
                                                view! { <span class="spinner border-on-ink/30 border-t-on-ink"></span> "Converting…" }.into_any()
                                            } else {
                                                view! {
                                                    {format!("Convert {}", plural(ready_count.get(), "file", "files"))}
                                                    <Icon icon=Ic::ArrowRight class="size-4"/>
                                                }.into_any()
                                            }}
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

// ── Empty state ─────────────────────────────────────

#[component]
fn EmptyState(open_picker: impl Fn() + Copy + Send + 'static) -> impl IntoView {
    let (mod_key, key_label) = if is_mac() {
        ("⌘", "Command")
    } else {
        ("Ctrl", "Control")
    };
    view! {
        <div class="p-2 sm:p-2.5">
            <div
                class="dropzone min-h-[17rem] sm:min-h-[21rem] px-6 py-10"
                role="button"
                tabindex="0"
                aria-label="Choose files to convert, or drop them here"
                on:click=move |_| open_picker()
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        open_picker();
                    }
                }
            >
                <span class="size-12 inline-flex items-center justify-center rounded-xl border border-line bg-surface text-accent-strong shadow-sm">
                    <Icon icon=Ic::Upload class="size-5"/>
                </span>
                <p class="mt-5 text-lg font-semibold tracking-tight">"Drop files to convert"</p>
                <p class="mt-1 text-sm text-muted">"Add one file or a whole batch."</p>
                <span class="btn btn-primary mt-6" aria-hidden="true">"Choose files"</span>
                <p class="hidden sm:flex items-center gap-1.5 mt-4 text-xs text-subtle">
                    "or paste with"
                    <kbd class="kbd" title=key_label>{mod_key}</kbd>
                    <kbd class="kbd">"V"</kbd>
                </p>
                <p class="mt-8 max-w-md text-xs leading-relaxed text-subtle text-balance">
                    "PNG, JPG, WebP, SVG, MP3, FLAC, Markdown, PDF, DOCX, CSV, JSON, YAML, TOML and more"
                </p>
            </div>
        </div>
    }
}

#[component]
fn DragOverlay(dragging: Memo<bool>) -> impl IntoView {
    view! {
        <Show when=move || dragging.get()>
            <div class="fixed inset-0 z-50 p-3 sm:p-6 bg-page/75 backdrop-blur-sm animate-fade-in pointer-events-none">
                <div class="size-full rounded-3xl border-2 border-dashed border-accent/70 bg-accent/[0.05] flex flex-col items-center justify-center text-center px-6">
                    <span class="size-14 inline-flex items-center justify-center rounded-2xl border border-line bg-surface text-accent-strong shadow-pop">
                        <Icon icon=Ic::Upload class="size-6"/>
                    </span>
                    <p class="mt-5 text-xl font-semibold tracking-tight">"Drop to add files"</p>
                    <p class="mt-1 text-sm text-muted">"They stay on this device."</p>
                </div>
            </div>
        </Show>
    }
}

// ── Options bar (convert-all target, quality) ───────

#[component]
fn OptionsBar(
    common_formats: Memo<Vec<&'static str>>,
    uniform_target: Memo<Option<String>>,
    set_all_targets: impl Fn(String) + 'static + Copy + Send + Sync,
    quality: RwSignal<u8>,
    show_quality: Memo<bool>,
    is_converting: RwSignal<bool>,
) -> impl IntoView {
    let visible = move || !common_formats.with(|c| c.is_empty()) || show_quality.get();
    view! {
        <Show when=visible>
            <div class="flex flex-wrap items-center gap-x-8 gap-y-3 px-4 sm:px-5 py-2.5 border-b border-line bg-sunken/50 text-[13px]">
                {move || {
                    let common = common_formats.get();
                    (!common.is_empty()).then(|| view! {
                        <label class="flex items-center gap-2.5">
                            <span class="text-muted">"Convert all to"</span>
                            <select
                                class="select"
                                prop:disabled=move || is_converting.get()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    if !val.is_empty() {
                                        set_all_targets(val);
                                    }
                                }
                            >
                                <option value="" disabled=true prop:selected=move || uniform_target.get().is_none()>
                                    "Mixed"
                                </option>
                                {common.into_iter().map(|fmt| view! {
                                    <option
                                        value=fmt
                                        prop:selected=move || uniform_target.get().as_deref() == Some(fmt)
                                    >
                                        {fmt.to_uppercase()}
                                    </option>
                                }).collect::<Vec<_>>()}
                            </select>
                        </label>
                    })
                }}

                {move || show_quality.get().then(|| view! {
                    <label class="flex items-center gap-3 flex-1 min-w-[15rem] max-w-sm">
                        <span class="text-muted whitespace-nowrap">"Quality"</span>
                        <input
                            type="range"
                            min="10"
                            max="100"
                            step="5"
                            class="range flex-1"
                            aria-label="Quality for JPG and AVIF output"
                            prop:value=move || quality.get().to_string()
                            prop:disabled=move || is_converting.get()
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                    quality.set(v);
                                }
                            }
                        />
                        <span class="w-8 text-right font-mono text-xs tabular-nums text-fg">{move || quality.get()}</span>
                    </label>
                })}
            </div>
        </Show>
    }
}

// ── File row ────────────────────────────────────────

#[component]
fn FileRow(
    id: usize,
    files: RwSignal<Vec<BatchFile>>,
    on_remove: impl Fn(usize) + 'static + Copy + Send,
    on_set_target: impl Fn(usize, String) + 'static + Copy + Send,
    on_save: impl Fn(usize) + 'static + Copy + Send,
) -> impl IntoView {
    // Static facts about the file, read once.
    let Some(initial) = files.with_untracked(|l| l.iter().find(|f| f.id == id).cloned()) else {
        return ().into_any();
    };
    let name = initial.name.clone();
    let input_size = initial.size;
    let cat = category_of(&initial.extension);
    let formats: Vec<&'static str> = converter::get_output_formats(&initial.extension);

    // Live view of this row's file; cheap to clone since bytes are shared.
    let file = Memo::new(move |_| files.with(|l| l.iter().find(|f| f.id == id).cloned()));
    let status = Memo::new(move |_| file.with(|f| f.as_ref().map(|f| f.status.clone())));
    let target = Memo::new(move |_| file.with(|f| f.as_ref().and_then(|f| f.target.clone())));
    let busy = move || matches!(status.get(), Some(FileStatus::Converting));
    // Once a batch has started, reserve room for the status so the format
    // selects line up across rows whatever state each row is in.
    let started =
        Memo::new(move |_| files.with(|l| l.iter().any(|f| f.status != FileStatus::Pending)));

    let meta = move || match status.get() {
        Some(FileStatus::Done { data, elapsed_ms }) => {
            let delta = format_size_delta(input_size, data.len()).map(|d| {
                let smaller = d.starts_with('-');
                let cls = if smaller { "text-ok" } else { "text-subtle" };
                view! { <span class=cls>{d.replace('-', "\u{2212}")}</span> }
            });
            view! {
                <span>{format_size(input_size)}</span>
                <Icon icon=Ic::ArrowRight class="size-3 shrink-0"/>
                <span class="text-muted">{format_size(data.len())}</span>
                {delta}
                <span aria-hidden="true">"·"</span>
                <span>{format_elapsed(elapsed_ms)}</span>
            }
            .into_any()
        }
        Some(FileStatus::Error(e)) => view! {
            <span class="text-bad truncate" title=e.clone()>{e.clone()}</span>
        }
        .into_any(),
        _ => view! { <span>{format_size(input_size)}</span> }.into_any(),
    };

    let status_slot = move || match status.get() {
        Some(FileStatus::Converting) => view! {
            <span class="inline-flex items-center gap-2 h-8 px-1 text-xs text-muted">
                <span class="spinner"></span>
                <span class="sm:sr-only">"Converting"</span>
            </span>
        }
        .into_any(),
        Some(FileStatus::Done { .. }) => view! {
            <button type="button" class="btn btn-sm btn-secondary" on:click=move |_| on_save(id)>
                <Icon icon=Ic::Download class="size-3.5"/>
                "Download"
            </button>
        }
        .into_any(),
        Some(FileStatus::Error(_)) => view! {
            <span class="inline-flex items-center gap-1.5 h-8 px-1 text-xs font-medium text-bad">
                <Icon icon=Ic::Alert class="size-4"/>
                "Failed"
            </span>
        }
        .into_any(),
        _ => ().into_any(),
    };

    view! {
        <li class="file-row">
            <CategoryTile cat=cat/>

            <div class="min-w-0">
                <p class="text-sm font-medium truncate" title=name.clone()>{name.clone()}</p>
                <p class="mt-0.5 flex items-center gap-1.5 min-w-0 text-xs text-subtle tabular-nums">
                    {meta}
                </p>
            </div>

            <div class="file-row-controls">
                <span class="text-xs text-subtle">"to"</span>
                <select
                    class="select"
                    aria-label=format!("Output format for {name}")
                    prop:disabled=busy
                    on:change=move |ev| on_set_target(id, event_target_value(&ev))
                >
                    {formats.into_iter().map(|fmt| view! {
                        <option value=fmt prop:selected=move || target.get().as_deref() == Some(fmt)>
                            {fmt.to_uppercase()}
                        </option>
                    }).collect::<Vec<_>>()}
                </select>

                <div class="flex" class=("w-[6.5rem]", move || started.get())>
                    {status_slot}
                </div>

                <button
                    type="button"
                    class="icon-btn"
                    prop:disabled=busy
                    aria-label=format!("Remove {name}")
                    title="Remove"
                    on:click=move |_| on_remove(id)
                >
                    <Icon icon=Ic::X class="size-4"/>
                </button>
            </div>
        </li>
    }
    .into_any()
}

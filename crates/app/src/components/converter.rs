use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

use crate::types::{BatchFile, FileStatus};
use crate::utils::{
    download_blob, download_blob_raw, format_elapsed, format_size, format_size_delta,
    make_output_name, next_tick,
};

/// Output formats that take a lossy quality setting.
fn is_lossy_target(fmt: &str) -> bool {
    matches!(fmt, "jpg" | "jpeg" | "avif")
}

#[component]
pub fn ConverterSection(
    files: RwSignal<Vec<BatchFile>>,
    next_id: RwSignal<usize>,
) -> impl IntoView {
    let is_converting = RwSignal::new(false);
    let quality = RwSignal::new(85u8);

    let has_files = Memo::new(move |_| !files.get().is_empty());
    let all_done = Memo::new(move |_| {
        let f = files.get();
        !f.is_empty() && f.iter().all(|f| f.status.is_finished())
    });
    let can_convert = Memo::new(move |_| {
        let f = files.get();
        !f.is_empty()
            && f.iter()
                .any(|f| f.target.is_some() && f.status == FileStatus::Pending)
            && !is_converting.get()
    });
    let done_count = Memo::new(move |_| {
        files
            .get()
            .iter()
            .filter(|f| matches!(f.status, FileStatus::Done { .. }))
            .count()
    });
    // Show the quality control when any queued file targets a lossy format.
    let show_quality = Memo::new(move |_| {
        files.get().iter().any(|f| {
            f.status == FileStatus::Pending && f.target.as_deref().is_some_and(is_lossy_target)
        })
    });
    // Output formats every file in the batch supports — drives "set all".
    let common_formats = Memo::new(move |_| {
        let f = files.get();
        if f.len() < 2 {
            return Vec::new();
        }
        let mut common: Vec<&'static str> = converter::get_output_formats(&f[0].extension);
        for file in &f[1..] {
            let fmts = converter::get_output_formats(&file.extension);
            common.retain(|c| fmts.contains(c));
        }
        common
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
                    bytes,
                    extension: ext,
                    size,
                    target: default_target,
                    status: FileStatus::Pending,
                });
            }
        });
    };

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
            let queue: Vec<usize> = files
                .get_untracked()
                .iter()
                .filter(|f| f.target.is_some() && f.status == FileStatus::Pending)
                .map(|f| f.id)
                .collect();

            for file_id in queue {
                let Some((bytes, ext, target_ext)) = files
                    .get_untracked()
                    .iter()
                    .find(|f| f.id == file_id)
                    .map(|f| (f.bytes.clone(), f.extension.clone(), f.target.clone()))
                else {
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
                            Ok(data) => FileStatus::Done { data, elapsed_ms },
                            Err(e) => FileStatus::Error(e),
                        };
                    }
                });
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
        let list = files.get_untracked();
        if let Some(f) = list.iter().find(|f| f.id == file_id)
            && let FileStatus::Done { ref data, .. } = f.status
        {
            let target_ext = f.target.as_deref().unwrap_or("bin");
            download_blob(data, &f.name, target_ext);
        }
    };

    let save_all_as = move |format: String| {
        let list = files.get_untracked();
        let entries: Vec<converter::archive::ArchiveEntry> = list
            .iter()
            .filter_map(|f| {
                if let FileStatus::Done { ref data, .. } = f.status {
                    let target_ext = f.target.as_deref().unwrap_or("bin");
                    Some(converter::archive::ArchiveEntry {
                        name: make_output_name(&f.name, target_ext),
                        data: data.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if entries.is_empty() {
            return;
        }

        let archive_result = match format.as_str() {
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

    view! {
        <section id="converter" class="pt-10 pb-8 sm:pt-14 sm:pb-10 h-full">
            <div class="plate h-full flex flex-col">
                <div class="flex items-center justify-between mb-4 pb-3 border-b hairline">
                    <span class="section-tag">"§ 01 · Workbench"</span>
                    {move || has_files.get().then(|| view! {
                        <button
                            class="text-[10px] uppercase tracking-[0.15em] font-bold text-base-content/40 hover:text-error transition-colors"
                            on:click=on_reset
                        >"[ Clear all ]"</button>
                    })}
                </div>

                <DropZone add_files=add_files has_files=has_files/>

                {move || {
                    let file_list = files.get();
                    if file_list.is_empty() {
                        return ().into_any();
                    }
                    view! {
                        <div class="mt-5 flex-1 min-h-0 flex flex-col">
                            // Ledger column headings
                            <div class="grid gap-x-3 px-3 pb-1.5 text-[10px] uppercase tracking-[0.2em] text-base-content/35 font-bold border-b-2 hairline"
                                style="grid-template-columns: 2rem 3.25rem minmax(0,1fr) auto">
                                <span>"Nº"</span>
                                <span>"Type"</span>
                                <span>"File"</span>
                                <span class="text-right">"Output"</span>
                            </div>

                            <div class="flex-1 min-h-0 overflow-y-auto">
                                <For
                                    each=move || files.get().into_iter().enumerate()
                                    key=|(_, f)| (
                                        f.id,
                                        f.target.clone(),
                                        match &f.status {
                                            FileStatus::Pending => 0u8,
                                            FileStatus::Converting => 1,
                                            FileStatus::Done { .. } => 2,
                                            FileStatus::Error(_) => 3,
                                        },
                                    )
                                    let:item
                                >
                                    <FileRow
                                        index=item.0
                                        file=item.1.clone()
                                        on_remove=remove_file
                                        on_set_target=set_target
                                        on_save=save_file
                                    />
                                </For>
                            </div>

                            <BatchControls
                                files=files
                                common_formats=common_formats
                                set_all_targets=set_all_targets
                                quality=quality
                                show_quality=show_quality
                                is_converting=is_converting
                                can_convert=can_convert
                                all_done=all_done
                                done_count=done_count
                                save_all_as=save_all_as
                                on_reset=on_reset
                                convert_all=convert_all
                            />
                        </div>
                    }
                    .into_any()
                }}
            </div>
        </section>
    }
}

// ── Batch controls (footer of the ledger) ───────────

#[component]
#[allow(clippy::too_many_arguments)]
fn BatchControls(
    files: RwSignal<Vec<BatchFile>>,
    common_formats: Memo<Vec<&'static str>>,
    set_all_targets: impl Fn(String) + 'static + Copy + Send,
    quality: RwSignal<u8>,
    show_quality: Memo<bool>,
    is_converting: RwSignal<bool>,
    can_convert: Memo<bool>,
    all_done: Memo<bool>,
    done_count: Memo<usize>,
    save_all_as: impl Fn(String) + 'static + Copy + Send,
    on_reset: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send,
    convert_all: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="pt-4 mt-1 space-y-3">
            {move || {
                let common = common_formats.get();
                (!common.is_empty()).then(|| view! {
                    <div class="flex items-center gap-3 text-xs">
                        <span class="uppercase tracking-[0.15em] text-base-content/40 font-bold">"Set all outputs"</span>
                        <select
                            class="select select-bordered select-xs bg-base-100"
                            prop:disabled=move || is_converting.get()
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if !val.is_empty() {
                                    set_all_targets(val);
                                }
                            }
                        >
                            <option value="" selected=true>"—"</option>
                            {common.into_iter().map(|fmt| view! {
                                <option value={fmt}>{fmt.to_uppercase()}</option>
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                })
            }}

            {move || show_quality.get().then(|| view! {
                <div class="flex items-center gap-3 text-xs">
                    <span class="uppercase tracking-[0.15em] text-base-content/40 font-bold whitespace-nowrap">
                        "Quality " <span class="text-primary tabular-nums">{move || quality.get()}</span>
                    </span>
                    <input
                        type="range"
                        min="10"
                        max="100"
                        step="5"
                        class="quality-range"
                        prop:value=move || quality.get().to_string()
                        prop:disabled=move || is_converting.get()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                                quality.set(v);
                            }
                        }
                    />
                </div>
            })}

            <div class="flex flex-col sm:flex-row gap-3">
                {move || {
                    if all_done.get() {
                        view! {
                            <div class="flex flex-col sm:flex-row gap-3 w-full">
                                <SaveAllDropdown done_count=done_count save_all_as=save_all_as/>
                                <button class="btn btn-ghost border hairline flex-1" on:click=on_reset>
                                    "Start over"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="btn btn-primary btn-lg w-full gap-3 group"
                                class:btn-disabled=move || !can_convert.get()
                                on:click=convert_all
                            >
                                {move || if is_converting.get() {
                                    let done = files.get().iter().filter(|f| f.status.is_finished()).count();
                                    let total = files.get().len();
                                    format!("Converting {done}/{total}…")
                                } else {
                                    let count = files.get().iter()
                                        .filter(|f| f.target.is_some() && f.status == FileStatus::Pending)
                                        .count();
                                    format!("Convert {count} file{}", if count != 1 { "s" } else { "" })
                                }}
                                <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                    <path d="M5 12h14M12 5l7 7-7 7"/>
                                </svg>
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

// ── File Row ────────────────────────────────────────

#[component]
fn FileRow(
    index: usize,
    file: BatchFile,
    on_remove: impl Fn(usize) + 'static + Copy + Send,
    on_set_target: impl Fn(usize, String) + 'static + Copy + Send,
    on_save: impl Fn(usize) + 'static + Copy + Send,
) -> impl IntoView {
    let file_id = file.id;
    let formats: Vec<String> = converter::get_output_formats(&file.extension)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let current_target = file.target.clone().unwrap_or_default();
    let input_size = file.size;
    let is_converting = matches!(file.status, FileStatus::Converting);

    let detail = match &file.status {
        FileStatus::Done { data, elapsed_ms } => {
            let mut s = format!("→ {}", format_size(data.len()));
            if let Some(delta) = format_size_delta(input_size, data.len()) {
                s.push_str(&format!(" ({delta})"));
            }
            s.push_str(&format!(" in {}", format_elapsed(*elapsed_ms)));
            Some((s, false))
        }
        FileStatus::Error(e) => Some((e.clone(), true)),
        _ => None,
    };

    view! {
        <div class="ledger-row group/row">
            <span class="ledger-index">{format!("{:02}", index + 1)}</span>

            <span class="ext-chip">{if file.extension.is_empty() { "?".to_string() } else { file.extension.clone() }}</span>

            <div class="min-w-0">
                <p class="text-sm truncate">{file.name.clone()}</p>
                <p class="text-[11px] text-base-content/40 tabular-nums">{format_size(file.size)}</p>
            </div>

            <div class="ledger-controls">
                <span class="text-base-content/25">"→"</span>
                <select
                    class="select select-bordered select-sm bg-base-100 min-w-[88px] text-xs"
                    prop:disabled=is_converting
                    on:change=move |ev| {
                        on_set_target(file_id, event_target_value(&ev));
                    }
                >
                    {formats.iter().map(|fmt| {
                        let selected = *fmt == current_target;
                        view! {
                            <option value={fmt.clone()} selected=selected>
                                {fmt.to_uppercase()}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>

                {match &file.status {
                    FileStatus::Pending => view! {
                        <span class="stamp stamp-idle hidden sm:inline-flex">"Queued"</span>
                    }.into_any(),
                    FileStatus::Converting => view! {
                        <span class="stamp stamp-busy">"Working"</span>
                    }.into_any(),
                    FileStatus::Done { .. } => view! {
                        <button
                            class="stamp stamp-done hover:bg-secondary hover:text-secondary-content transition-colors"
                            on:click=move |_| on_save(file_id)
                        >
                            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                <polyline points="7 10 12 15 17 10"/>
                                <line x1="12" y1="15" x2="12" y2="3"/>
                            </svg>
                            "Save"
                        </button>
                    }.into_any(),
                    FileStatus::Error(_) => view! {
                        <span class="stamp stamp-error">"Failed"</span>
                    }.into_any(),
                }}

                <button
                    class="text-base-content/25 hover:text-error transition-colors px-1 disabled:opacity-30"
                    prop:disabled=is_converting
                    aria-label="Remove file"
                    on:click=move |_| on_remove(file_id)
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18"/>
                        <line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                </button>
            </div>

            {detail.map(|(text, is_error)| {
                let cls = if is_error {
                    "col-span-full text-[11px] tabular-nums sm:pl-[5.5rem] pt-0.5 text-error"
                } else {
                    "col-span-full text-[11px] tabular-nums sm:pl-[5.5rem] pt-0.5 text-secondary"
                };
                view! { <p class=cls>{text}</p> }
            })}
        </div>
    }
}

// ── Save All Dropdown ───────────────────────────────

#[component]
fn SaveAllDropdown(
    done_count: Memo<usize>,
    save_all_as: impl Fn(String) + 'static + Copy + Send,
) -> impl IntoView {
    const ARCHIVE_OPTIONS: [(&str, &str); 4] = [
        ("zip", "Most compatible"),
        ("tar.gz", "Smaller, Unix-native"),
        ("tar.xz", "Best compression"),
        ("7z", "High compression, widely supported"),
    ];

    view! {
        <div class="dropdown dropdown-top flex-1 w-full">
            <div tabindex="0" role="button" class="btn btn-secondary w-full gap-2 text-sm sm:text-base">
                <svg class="w-5 h-5 hidden sm:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7 10 12 15 17 10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                </svg>
                "Save " {move || done_count.get()} " files"
                <svg class="w-4 h-4 ml-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <polyline points="18 15 12 9 6 15"/>
                </svg>
            </div>
            <ul tabindex="0" class="dropdown-content z-[1] menu p-2 bg-base-200 w-full mb-2 border hairline shadow-2xl">
                {ARCHIVE_OPTIONS.into_iter().map(|(fmt, desc)| view! {
                    <li><a on:click=move |_| save_all_as(fmt.to_string())>
                        <span class="font-bold uppercase tracking-wider text-xs">{fmt}</span>
                        <span class="text-xs text-base-content/40">{desc}</span>
                    </a></li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

// ── Drop Zone ───────────────────────────────────────

#[component]
fn DropZone(
    add_files: impl Fn(Vec<(String, Vec<u8>)>) + 'static + Clone,
    has_files: Memo<bool>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    // Counter rather than bool: dragenter/dragleave also fire when moving
    // over child elements, and a bool flickers off mid-drag.
    let drag_depth = RwSignal::new(0i32);
    let dragging = Memo::new(move |_| drag_depth.get() > 0);

    let accept_str: String = converter::ALL_INPUT_FORMATS
        .iter()
        .map(|ext| format!(".{ext}"))
        .collect::<Vec<_>>()
        .join(",");

    let process_file_list = {
        let add_files = add_files.clone();
        move |file_list: web_sys::FileList| {
            let add = add_files.clone();
            let count = file_list.length();
            let collected = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let remaining = std::rc::Rc::new(std::cell::Cell::new(count));

            for i in 0..count {
                if let Some(file) = file_list.get(i) {
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
        }
    };

    let process_drop = process_file_list.clone();
    let process_input = process_file_list;

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        drag_depth.set(0);
        if let Some(dt) = ev.data_transfer()
            && let Some(files) = dt.files()
        {
            process_drop(files);
        }
    };

    let on_change = move |_ev: leptos::ev::Event| {
        if let Some(input) = input_ref.get() {
            let el: &web_sys::HtmlInputElement = &input;
            if let Some(files) = el.files() {
                process_input(files);
            }
            el.set_value("");
        }
    };

    let on_browse = move |_| {
        if let Some(input) = input_ref.get() {
            let el: &web_sys::HtmlInputElement = &input;
            el.click();
        }
    };

    view! {
        <div
            class="drop-slab"
            class=("drop-slab-active", move || dragging.get())
            class=("p-4", move || has_files.get())
            class=("p-10", move || !has_files.get())
            class=("sm:p-14", move || !has_files.get())
            on:dragover=move |ev: web_sys::DragEvent| { ev.prevent_default(); }
            on:dragenter=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                drag_depth.update(|d| *d += 1);
            }
            on:dragleave=move |_: web_sys::DragEvent| {
                drag_depth.update(|d| *d = (*d - 1).max(0));
            }
            on:drop=on_drop
            on:click=on_browse
        >
            <span class="corner corner-tl"></span>
            <span class="corner corner-tr"></span>
            <span class="corner corner-bl"></span>
            <span class="corner corner-br"></span>

            <input
                node_ref=input_ref
                type="file"
                multiple=true
                accept={accept_str}
                class="hidden"
                on:change=on_change
            />

            {move || {
                if has_files.get() {
                    view! {
                        <div class="flex items-center justify-center gap-3 text-center">
                            <span class="text-primary text-lg leading-none">"+"</span>
                            <span class="text-xs uppercase tracking-[0.15em] text-base-content/50">
                                "Drop more files or " <span class="text-primary">"browse"</span>
                            </span>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col items-center gap-3 text-center select-none">
                            <svg class="w-10 h-10 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                <polyline points="17 8 12 3 7 8"/>
                                <line x1="12" y1="3" x2="12" y2="15"/>
                            </svg>
                            <div>
                                <p class="text-base font-bold uppercase tracking-[0.2em]">"Drop files here"</p>
                                <p class="text-xs text-base-content/50 mt-2 uppercase tracking-[0.12em]">
                                    "or " <span class="text-primary">"browse"</span> " · batches welcome"
                                </p>
                            </div>
                            <p class="text-[10px] text-base-content/30 uppercase tracking-[0.25em] mt-1">
                                "Images · Audio · Documents · Data · Config"
                            </p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

use crate::types::{BatchFile, FileStatus};
use crate::utils::{download_blob, download_blob_raw, format_icon, format_size, make_output_name};

#[component]
pub fn ConverterSection(
    files: RwSignal<Vec<BatchFile>>,
    next_id: RwSignal<usize>,
) -> impl IntoView {
    let dragging = RwSignal::new(false);
    let is_converting = RwSignal::new(false);

    let has_files = Memo::new(move |_| !files.get().is_empty());
    let all_done = Memo::new(move |_| {
        let f = files.get();
        !f.is_empty()
            && f.iter()
                .all(|f| matches!(f.status, FileStatus::Done(_) | FileStatus::Error(_)))
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
            .filter(|f| matches!(f.status, FileStatus::Done(_)))
            .count()
    });

    let add_files = move |new_files: Vec<(String, Vec<u8>)>| {
        files.update(|list| {
            for (name, bytes) in new_files {
                let size = bytes.len();
                let ext = converter::detect_format(&name).unwrap_or_default();
                let formats = converter::get_output_formats(&ext);
                let default_target = formats.first().map(|s| s.to_string());
                let id = next_id.get();
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
                // Allow re-converting: if the file was already done or errored,
                // reset it to Pending so the Convert button re-appears.
                if matches!(f.status, FileStatus::Done(_) | FileStatus::Error(_)) {
                    f.status = FileStatus::Pending;
                }
            }
        });
    };

    let convert_all = move |_| {
        is_converting.set(true);
        let file_list = files.get();

        for file in file_list.iter() {
            if file.target.is_none() || file.status != FileStatus::Pending {
                continue;
            }

            let file_id = file.id;
            let ext = file.extension.clone();
            let bytes = file.bytes.clone();
            let target_ext = file.target.clone().unwrap();

            files.update(|list| {
                if let Some(f) = list.iter_mut().find(|f| f.id == file_id) {
                    f.status = FileStatus::Converting;
                }
            });

            let files_signal = files;
            let is_converting_signal = is_converting;
            wasm_bindgen_futures::spawn_local(async move {
                let _ = JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL)).await;

                let config = format!(r#"{{"from":"{}","to":"{}"}}"#, ext, target_ext);
                let result = converter::convert(&bytes, &config);

                files_signal.update(|list| {
                    if let Some(f) = list.iter_mut().find(|f| f.id == file_id) {
                        match result {
                            Ok(output) => f.status = FileStatus::Done(output),
                            Err(e) => f.status = FileStatus::Error(e),
                        }
                    }
                });

                let all_finished = files_signal
                    .get()
                    .iter()
                    .all(|f| !matches!(f.status, FileStatus::Converting));
                if all_finished {
                    is_converting_signal.set(false);
                }
            });
        }
    };

    let on_reset = move |_| {
        files.set(Vec::new());
        is_converting.set(false);
    };

    let save_file = move |file_id: usize| {
        let list = files.get();
        if let Some(f) = list.iter().find(|f| f.id == file_id)
            && let FileStatus::Done(ref output) = f.status
        {
            let target_ext = f.target.as_deref().unwrap_or("bin");
            download_blob(output, &f.name, target_ext);
        }
    };

    let save_all_as = move |format: String| {
        let list = files.get();
        let entries: Vec<converter::archive::ArchiveEntry> = list
            .iter()
            .filter_map(|f| {
                if let FileStatus::Done(ref output) = f.status {
                    let target_ext = f.target.as_deref().unwrap_or("bin");
                    let output_name = make_output_name(&f.name, target_ext);
                    Some(converter::archive::ArchiveEntry {
                        name: output_name,
                        data: output.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        if entries.is_empty() {
            return;
        }

        let (archive_data, archive_ext, mime) = match format.as_str() {
            "zip" => match converter::archive::create_zip(&entries) {
                Ok(data) => (data, "zip", "application/zip"),
                Err(e) => {
                    web_sys::console::error_1(&format!("Archive error: {e}").into());
                    return;
                }
            },
            "tar.gz" => match converter::archive::create_tar_gz(&entries) {
                Ok(data) => (data, "tar.gz", "application/gzip"),
                Err(e) => {
                    web_sys::console::error_1(&format!("Archive error: {e}").into());
                    return;
                }
            },
            "tar.xz" => match converter::archive::create_tar_xz(&entries) {
                Ok(data) => (data, "tar.xz", "application/x-xz"),
                Err(e) => {
                    web_sys::console::error_1(&format!("Archive error: {e}").into());
                    return;
                }
            },
            "7z" => match converter::archive::create_7z(&entries) {
                Ok(data) => (data, "7z", "application/x-7z-compressed"),
                Err(e) => {
                    web_sys::console::error_1(&format!("Archive error: {e}").into());
                    return;
                }
            },
            _ => return,
        };

        download_blob_raw(
            &archive_data,
            &format!("transfigure-output.{archive_ext}"),
            mime,
        );
    };

    view! {
        <section id="converter" class="pt-10 pb-8 sm:pt-14 sm:pb-10 h-full">
            <div class="panel-shell h-full flex flex-col">
                <DropZone dragging=dragging add_files=add_files has_files=has_files/>

                {move || {
                    let file_list = files.get();
                    if file_list.is_empty() {
                        view! { <div class="hidden"></div> }.into_any()
                    } else {
                        let is_conv = is_converting.get();
                        view! {
                            <div class="mt-6 flex-1 min-h-0 flex flex-col">
                                <div class="flex items-center justify-between mb-2">
                                    <span class="text-sm text-base-content/50 font-medium">
                                        {file_list.len()} " file" {if file_list.len() != 1 { "s" } else { "" }}
                                    </span>
                                    <button
                                        class="btn btn-ghost btn-xs text-base-content/40 hover:text-error"
                                        on:click=on_reset
                                    >"Clear all"</button>
                                </div>

                                <div class="space-y-3 flex-1 min-h-0 overflow-y-auto pr-1">
                                    <For
                                        each=move || files.get()
                                        key=|f| f.id
                                        let:file
                                    >
                                        <FileRow
                                            file=file.clone()
                                            on_remove=remove_file
                                            on_set_target=set_target
                                            on_save=save_file
                                        />
                                    </For>
                                </div>

                                <div class="flex flex-col sm:flex-row gap-3 pt-4 mt-4 border-t border-white/5">
                                    {move || {
                                        if all_done.get() {
                                            view! {
                                                <div class="flex flex-col sm:flex-row gap-3 w-full">
                                                    <SaveAllDropdown done_count=done_count save_all_as=save_all_as/>
                                                    <button class="btn btn-ghost flex-1" on:click=on_reset>
                                                        "Start over"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <button
                                                    class="btn btn-primary btn-lg w-full gap-2 group"
                                                    class:btn-disabled=move || !can_convert.get()
                                                    on:click=convert_all
                                                >
                                                    {move || if is_conv {
                                                        "Converting...".to_string()
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
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}

// ── File Row ────────────────────────────────────────

#[component]
fn FileRow(
    file: BatchFile,
    on_remove: impl Fn(usize) + 'static + Copy,
    on_set_target: impl Fn(usize, String) + 'static + Copy,
    on_save: impl Fn(usize) + 'static + Copy,
) -> impl IntoView {
    let file_id = file.id;
    let formats: Vec<String> = converter::get_output_formats(&file.extension)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let current_target = file.target.clone().unwrap_or_default();
    let is_done = matches!(file.status, FileStatus::Done(_));
    let is_converting = matches!(file.status, FileStatus::Converting);
    let is_error = matches!(file.status, FileStatus::Error(_));
    let error_msg = if let FileStatus::Error(ref e) = file.status {
        e.clone()
    } else {
        String::new()
    };

    view! {
        <div class=move || {
            let base = "flex flex-col gap-2 panel-row transition-all duration-200";
            if is_done {
                format!("{base} bg-success/5 border-success/20")
            } else if is_error {
                format!("{base} bg-error/5 border-error/20")
            } else if is_converting {
                format!("{base} bg-primary/5 border-primary/20")
            } else {
                format!("{base} bg-base-100/50 border-white/5")
            }
        }>
            <div class="flex items-center gap-3">
                <span class="text-xl flex-shrink-0">{format_icon(&file.extension)}</span>

                <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium truncate">{file.name.clone()}</p>
                    <p class="text-xs text-base-content/40">
                        {format_size(file.size)}
                        " · "
                        <span class="uppercase font-medium text-base-content/50">{file.extension.clone()}</span>
                    </p>
                </div>

                <svg class="w-4 h-4 text-base-content/20 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <path d="M5 12h14M12 5l7 7-7 7"/>
                </svg>

                <select
                    class="select select-sm select-bordered bg-base-100/80 min-w-[100px] text-sm font-medium"
                    prop:disabled=is_converting
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        on_set_target(file_id, val);
                    }
                >
                    {formats.iter().map(|fmt| {
                        let selected = *fmt == current_target;
                        let fmt_val = fmt.clone();
                        view! {
                            <option value={fmt_val} selected=selected>
                                {fmt.to_uppercase()}
                            </option>
                        }
                    }).collect::<Vec<_>>()}
                </select>

                <div class="flex items-center gap-1 flex-shrink-0">
                    {if is_done {
                        view! {
                            <button
                                class="btn btn-success btn-sm gap-1"
                                on:click=move |_| on_save(file_id)
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                                    <polyline points="17 21 17 13 7 13 7 21"/>
                                    <polyline points="7 3 7 8 15 8"/>
                                </svg>
                                "Save"
                            </button>
                        }.into_any()
                    } else if is_converting {
                        view! {
                            <span class="loading loading-spinner loading-sm text-primary"></span>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="btn btn-ghost btn-sm btn-circle text-base-content/30 hover:text-error"
                                on:click=move |_| on_remove(file_id)
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                    <line x1="18" y1="6" x2="6" y2="18"/>
                                    <line x1="6" y1="6" x2="18" y2="18"/>
                                </svg>
                            </button>
                        }.into_any()
                    }}
                </div>
            </div>

            {if is_error {
                view! {
                    <p class="text-xs text-error pl-8">{error_msg.clone()}</p>
                }.into_any()
            } else {
                view! { <span class="hidden"></span> }.into_any()
            }}
        </div>
    }
}

// ── Save All Dropdown ───────────────────────────────

#[component]
fn SaveAllDropdown(
    done_count: Memo<usize>,
    save_all_as: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let save_zip = {
        let cb = save_all_as.clone();
        move |_: web_sys::MouseEvent| cb("zip".to_string())
    };
    let save_tar_gz = {
        let cb = save_all_as.clone();
        move |_: web_sys::MouseEvent| cb("tar.gz".to_string())
    };
    let save_tar_xz = {
        let cb = save_all_as.clone();
        move |_: web_sys::MouseEvent| cb("tar.xz".to_string())
    };
    let save_7z = {
        let cb = save_all_as.clone();
        move |_: web_sys::MouseEvent| cb("7z".to_string())
    };

    view! {
        <div class="dropdown dropdown-top flex-1 w-full">
            <div tabindex="0" role="button" class="btn btn-success w-full gap-2 text-sm sm:text-base">
                <svg class="w-5 h-5 hidden sm:block" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                    <polyline points="17 21 17 13 7 13 7 21"/>
                    <polyline points="7 3 7 8 15 8"/>
                </svg>
                "Save " {move || done_count.get()} " files"
                <svg class="w-4 h-4 ml-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <polyline points="18 15 12 9 6 15"/>
                </svg>
            </div>
            <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow-2xl bg-base-200 rounded-box w-full mb-2 border border-white/10">
                <li><a on:click=save_zip>
                    <span class="font-medium">"ZIP"</span>
                    <span class="text-xs text-base-content/40">"Most compatible"</span>
                </a></li>
                <li><a on:click=save_tar_gz>
                    <span class="font-medium">"TAR.GZ"</span>
                    <span class="text-xs text-base-content/40">"Smaller, Unix-native"</span>
                </a></li>
                <li><a on:click=save_tar_xz>
                    <span class="font-medium">"TAR.XZ"</span>
                    <span class="text-xs text-base-content/40">"Best compression"</span>
                </a></li>
                <li><a on:click=save_7z>
                    <span class="font-medium">"7Z"</span>
                    <span class="text-xs text-base-content/40">"High compression, widely supported"</span>
                </a></li>
            </ul>
        </div>
    }
}

// ── Drop Zone ───────────────────────────────────────

#[component]
fn DropZone(
    dragging: RwSignal<bool>,
    add_files: impl Fn(Vec<(String, Vec<u8>)>) + 'static + Clone,
    has_files: Memo<bool>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

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
                            let bytes = array.to_vec();
                            collected.borrow_mut().push((name, bytes));
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
        dragging.set(false);
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
            class=move || {
                let compact = has_files.get();
                if dragging.get() {
                    if compact {
                        "relative rounded-xl border-2 border-dashed p-4 text-center transition-all duration-300 cursor-pointer border-primary bg-primary/10 scale-[1.02]"
                    } else {
                        "relative rounded-xl border-2 border-dashed p-12 sm:p-16 text-center transition-all duration-300 cursor-pointer border-primary bg-primary/10 scale-105"
                    }
                } else if compact {
                    "relative rounded-xl border-2 border-dashed p-4 text-center transition-all duration-300 cursor-pointer border-base-content/10 hover:border-primary/40 hover:bg-primary/5"
                } else {
                    "relative rounded-xl border-2 border-dashed p-12 sm:p-16 text-center transition-all duration-300 cursor-pointer border-base-content/10 hover:border-primary/40 hover:bg-primary/5"
                }
            }
            on:dragover=move |ev: web_sys::DragEvent| { ev.prevent_default(); dragging.set(true); }
            on:dragenter=move |ev: web_sys::DragEvent| { ev.prevent_default(); dragging.set(true); }
            on:dragleave=move |_: web_sys::DragEvent| { dragging.set(false); }
            on:drop=on_drop
            on:click=on_browse
        >
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
                        <div class="flex items-center justify-center gap-3">
                            <svg class="w-5 h-5 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                <line x1="12" y1="5" x2="12" y2="19"/>
                                <line x1="5" y1="12" x2="19" y2="12"/>
                            </svg>
                            <span class="text-sm text-base-content/50">"Drop more files or "<span class="text-primary underline underline-offset-2">"browse"</span></span>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col items-center gap-4">
                            <div class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center">
                                <svg class="w-8 h-8 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                    <polyline points="17 8 12 3 7 8"/>
                                    <line x1="12" y1="3" x2="12" y2="15"/>
                                </svg>
                            </div>
                            <div>
                                <p class="text-lg font-medium">"Drop your files here"</p>
                                <p class="text-sm text-base-content/50 mt-1">
                                    "or "
                                    <span class="text-primary underline underline-offset-2">"browse files"</span>
                                    " · Multiple files supported"
                                </p>
                            </div>
                            <p class="text-xs text-base-content/30">"Images · Audio · Documents · Data · Config"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

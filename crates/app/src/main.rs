use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

// ── State types ────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum FileStatus {
    Pending,
    Converting,
    Done(Vec<u8>),
    Error(String),
}

#[derive(Clone)]
struct BatchFile {
    id: usize,
    name: String,
    bytes: Vec<u8>,
    extension: String,
    size: usize,
    target: Option<String>,
    status: FileStatus,
}

// ── Entry point ────────────────────────────────────────────

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

// ── Root component ─────────────────────────────────────────

#[component]
fn App() -> impl IntoView {
    let files = RwSignal::new(Vec::<BatchFile>::new());
    let next_id = RwSignal::new(0usize);

    view! {
        <div class="min-h-screen bg-base-300 relative overflow-hidden">
            <div class="pointer-events-none fixed inset-0 z-0">
                <div class="orb orb-1"></div>
                <div class="orb orb-2"></div>
                <div class="orb orb-3"></div>
                <div class="noise"></div>
            </div>

            <div class="relative z-10">
                <Header/>

                <main class="container mx-auto px-4 sm:px-6 lg:px-8 pb-20">
                    <Hero/>
                    <ConverterSection files=files next_id=next_id/>
                    <HowItWorks/>
                    <Formats/>
                    <PrivacyBanner/>
                </main>

                <Footer/>
            </div>
        </div>
    }
}

// ── Header ──────────────────────────────────────────

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 backdrop-blur-xl bg-base-300/70 border-b border-white/5">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between h-16">
                <a href="/" class="flex items-center gap-2 text-lg font-bold tracking-tight">
                    <span class="text-2xl">"⚗️"</span>
                    <span class="gradient-text">"Transfigure"</span>
                </a>
                <nav class="flex items-center gap-6">
                    <a href="#how" class="text-sm text-base-content/60 hover:text-base-content transition-colors">"How it works"</a>
                    <a href="#formats" class="text-sm text-base-content/60 hover:text-base-content transition-colors">"Formats"</a>
                    <a
                        href="https://github.com/RephlexZero/transfigure"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-ghost btn-sm btn-circle"
                    >
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
                        </svg>
                    </a>
                </nav>
            </div>
        </header>
    }
}

// ── Hero ────────────────────────────────────────────

#[component]
fn Hero() -> impl IntoView {
    view! {
        <section class="pt-20 pb-12 sm:pt-28 sm:pb-16 text-center max-w-3xl mx-auto">
            <div class="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-primary/10 border border-primary/20 text-sm text-primary mb-8">
                <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                </span>
                "100% Client-Side · Zero Uploads"
            </div>

            <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight leading-tight mb-6">
                "Convert files."
                <br/>
                <span class="gradient-text">"Without the trust issues."</span>
            </h1>

            <p class="text-lg sm:text-xl text-base-content/60 max-w-2xl mx-auto leading-relaxed">
                "Your files never leave your browser. Everything runs locally via "
                <span class="text-accent font-medium">"WebAssembly"</span>
                " at near-native speed. No servers. No uploads. No BS."
            </p>
        </section>
    }
}

// ── Converter Section ───────────────────────────────

#[component]
fn ConverterSection(files: RwSignal<Vec<BatchFile>>, next_id: RwSignal<usize>) -> impl IntoView {
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
            _ => return,
        };

        download_blob_raw(
            &archive_data,
            &format!("transfigure-output.{archive_ext}"),
            mime,
        );
    };

    view! {
        <section id="converter" class="max-w-3xl mx-auto mb-24">
            <div class="glass-card rounded-2xl p-6 sm:p-8">
                <DropZone dragging=dragging add_files=add_files has_files=has_files/>

                {move || {
                    let file_list = files.get();
                    if file_list.is_empty() {
                        view! { <div class="hidden"></div> }.into_any()
                    } else {
                        let is_conv = is_converting.get();
                        view! {
                            <div class="mt-6 space-y-3">
                                <div class="flex items-center justify-between mb-2">
                                    <span class="text-sm text-base-content/50 font-medium">
                                        {file_list.len()} " file" {if file_list.len() != 1 { "s" } else { "" }}
                                    </span>
                                    <button
                                        class="btn btn-ghost btn-xs text-base-content/40 hover:text-error"
                                        on:click=on_reset
                                    >"Clear all"</button>
                                </div>

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

                                <div class="flex flex-col sm:flex-row gap-3 pt-4 border-t border-white/5">
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
            let base = "flex flex-col gap-2 p-3 rounded-xl border transition-all duration-200";
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
                    prop:disabled=is_done || is_converting
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

    view! {
        <div class="dropdown dropdown-top flex-1">
            <div tabindex="0" role="button" class="btn btn-success btn-lg w-full gap-2">
                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                    <polyline points="17 21 17 13 7 13 7 21"/>
                    <polyline points="7 3 7 8 15 8"/>
                </svg>
                "Save all " {move || done_count.get()} " files"
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
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
                            <p class="text-xs text-base-content/30">"Images · Documents · Data · Config files"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ── How It Works ────────────────────────────────────

#[component]
fn HowItWorks() -> impl IntoView {
    view! {
        <section id="how" class="py-20 max-w-5xl mx-auto">
            <h2 class="text-3xl sm:text-4xl font-bold text-center mb-14">
                "How it "
                <span class="gradient-text">"works"</span>
            </h2>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <StepCard
                    number="01"
                    icon="🔒"
                    title="Drop your files"
                    description="Your files are read into browser memory via the File API. They never touch any server. Batch processing? Just drop them all at once."
                />
                <StepCard
                    number="02"
                    icon="⚡"
                    title="WASM converts them"
                    description="Our Rust-compiled WebAssembly engine processes every file at near-native speed, entirely in your browser."
                />
                <StepCard
                    number="03"
                    icon="💾"
                    title="Save the results"
                    description="Save files individually or bundle everything into a ZIP, TAR.GZ, or TAR.XZ archive. All generated locally."
                />
            </div>
        </section>
    }
}

#[component]
fn StepCard(
    number: &'static str,
    icon: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="glass-card rounded-2xl p-6 group hover:border-primary/30 transition-all duration-300">
            <div class="flex items-center justify-between mb-4">
                <span class="text-5xl">{icon}</span>
                <span class="text-4xl font-black text-base-content/5 group-hover:text-primary/10 transition-colors">{number}</span>
            </div>
            <h3 class="text-lg font-bold mb-2">{title}</h3>
            <p class="text-sm text-base-content/50 leading-relaxed">{description}</p>
        </div>
    }
}

// ── Formats ─────────────────────────────────────────

#[component]
fn Formats() -> impl IntoView {
    view! {
        <section id="formats" class="py-20 max-w-5xl mx-auto">
            <h2 class="text-3xl sm:text-4xl font-bold text-center mb-14">
                "Supported "
                <span class="gradient-text">"formats"</span>
            </h2>

            <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                <FormatCard icon="🖼️" title="Images" formats="PNG, JPG, WebP, GIF, BMP, TIFF"/>
                <FormatCard icon="✏️" title="Documents" formats="Markdown ↔ HTML, Markdown → Text"/>
                <FormatCard icon="📊" title="Data" formats="CSV ↔ JSON, CSV ↔ TSV"/>
                <FormatCard icon="⚙️" title="Config" formats="JSON ↔ YAML, JSON ↔ TOML"/>
                <FormatCard icon="🔐" title="Encoding" formats="Base64 ↔ Binary"/>
                <FormatCard icon="🎨" title="Vector" formats="SVG → PNG"/>
            </div>
        </section>
    }
}

#[component]
fn FormatCard(icon: &'static str, title: &'static str, formats: &'static str) -> impl IntoView {
    view! {
        <div class="glass-card rounded-xl p-5 text-center hover:border-primary/20 transition-all duration-300 group">
            <span class="text-3xl mb-3 block group-hover:scale-110 transition-transform">{icon}</span>
            <h3 class="font-bold text-sm mb-1">{title}</h3>
            <p class="text-xs text-base-content/40">{formats}</p>
        </div>
    }
}

// ── Privacy Banner ──────────────────────────────────

#[component]
fn PrivacyBanner() -> impl IntoView {
    view! {
        <section class="py-20 max-w-3xl mx-auto text-center">
            <div class="glass-card rounded-2xl p-8 sm:p-12 border-primary/10">
                <h2 class="text-2xl sm:text-3xl font-bold mb-4">
                    "Privacy isn't a policy."
                    <br/>
                    <span class="gradient-text">"It's the architecture."</span>
                </h2>
                <p class="text-base-content/50 leading-relaxed mb-8 max-w-xl mx-auto">
                    "Open your browser's DevTools. Watch the Network tab during a conversion. You'll see exactly zero upload requests. That's not because we promise to delete your files — it's because your files never leave your device in the first place."
                </p>
                <div class="flex flex-wrap justify-center gap-2">
                    <span class="badge badge-lg badge-outline badge-primary gap-1">"🚫 No uploads"</span>
                    <span class="badge badge-lg badge-outline badge-secondary gap-1">"🕵️ No tracking"</span>
                    <span class="badge badge-lg badge-outline badge-accent gap-1">"📡 Works offline"</span>
                    <span class="badge badge-lg badge-outline gap-1">"🔓 Open source"</span>
                </div>
            </div>
        </section>
    }
}

// ── Footer ──────────────────────────────────────────

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-white/5 py-8">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-base-content/40">
                <div class="flex items-center gap-2">
                    <span>"⚗️"</span>
                    <span class="font-medium">"Transfigure"</span>
                </div>
                <p>"Built with Rust & WebAssembly. Your files stay yours."</p>
                <a
                    href="https://github.com/RephlexZero/transfigure"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="hover:text-base-content transition-colors"
                >"GitHub"</a>
            </div>
        </footer>
    }
}

// ── Utility functions ───────────────────────────────

fn format_icon(ext: &str) -> &'static str {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "svg" => "🖼️",
        "md" | "markdown" | "html" | "txt" => "📝",
        "csv" | "tsv" | "json" => "📊",
        "yaml" | "yml" | "toml" => "⚙️",
        "base64" => "🔐",
        _ => "📄",
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn mime_type_for(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        "html" => "text/html",
        "md" | "markdown" => "text/markdown",
        "txt" | "text" => "text/plain",
        "csv" => "text/csv",
        "tsv" => "text/tab-separated-values",
        "json" => "application/json",
        "yaml" | "yml" => "application/x-yaml",
        "toml" => "application/toml",
        _ => "application/octet-stream",
    }
}

fn make_output_name(original_name: &str, target_ext: &str) -> String {
    if let Some((stem, _)) = original_name.rsplit_once('.') {
        format!("{stem}.{target_ext}")
    } else {
        format!("{original_name}.{target_ext}")
    }
}

fn download_blob(data: &[u8], original_name: &str, target_ext: &str) {
    let output_name = make_output_name(original_name, target_ext);
    download_blob_raw(data, &output_name, mime_type_for(target_ext));
}

fn download_blob_raw(data: &[u8], filename: &str, mime: &str) {
    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let a = document.create_element("a").unwrap();
    a.set_attribute("href", &url).unwrap();
    a.set_attribute("download", filename).unwrap();

    let el: &web_sys::HtmlElement = a.unchecked_ref();
    el.click();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

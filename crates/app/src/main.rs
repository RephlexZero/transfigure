use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

// ── State types ────────────────────────────────────────────

#[derive(Clone)]
struct FileData {
    name: String,
    bytes: Vec<u8>,
    extension: String,
    size: usize,
}

#[derive(Clone, PartialEq)]
enum AppPhase {
    Idle,
    FileSelected,
    Converting,
    Done { output: Vec<u8>, target_ext: String, original_name: String },
    Error(String),
}

// ── Entry point ────────────────────────────────────────────

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

// ── Root component ─────────────────────────────────────────

#[component]
fn App() -> impl IntoView {
    let file = RwSignal::new(Option::<FileData>::None);
    let target = RwSignal::new(Option::<String>::None);
    let phase = RwSignal::new(AppPhase::Idle);

    view! {
        <div class="min-h-screen bg-base-300 relative overflow-hidden">
            // Animated background
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
                    <ConverterSection file=file target=target phase=phase/>
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
                        href="https://github.com/RephlexZero/transfigureing"
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
fn ConverterSection(
    file: RwSignal<Option<FileData>>,
    target: RwSignal<Option<String>>,
    phase: RwSignal<AppPhase>,
) -> impl IntoView {
    let dragging = RwSignal::new(false);

    let on_file_selected = move |name: String, bytes: Vec<u8>| {
        let size = bytes.len();
        let ext = converter::detect_format(&name).unwrap_or_default();
        file.set(Some(FileData { name, bytes, extension: ext, size }));
        target.set(None);
        phase.set(AppPhase::FileSelected);
    };

    let on_convert = move |_| {
        let Some(f) = file.get() else { return };
        let Some(t) = target.get() else { return };

        phase.set(AppPhase::Converting);
        let original_name = f.name.clone();
        let ext = f.extension.clone();
        let bytes = f.bytes.clone();
        let target_ext = t.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // Yield to let UI paint the Converting state
            let _ = JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL)).await;

            let config = format!(r#"{{"from":"{}","to":"{}"}}"#, ext, target_ext);
            match converter::convert(&bytes, &config) {
                Ok(output) => {
                    phase.set(AppPhase::Done { output, target_ext, original_name });
                }
                Err(e) => {
                    phase.set(AppPhase::Error(e));
                }
            }
        });
    };

    let on_reset = move |_| {
        file.set(None);
        target.set(None);
        phase.set(AppPhase::Idle);
    };

    let on_download = move |_| {
        if let AppPhase::Done { ref output, ref target_ext, ref original_name } = phase.get() {
            download_blob(&output, original_name, target_ext);
        }
    };

    view! {
        <section id="converter" class="max-w-2xl mx-auto mb-24">
            <div class="glass-card rounded-2xl p-6 sm:p-8">
                {move || {
                    match phase.get() {
                        AppPhase::Idle => view! {
                            <DropZone dragging=dragging on_file_selected=on_file_selected.clone() />
                        }.into_any(),

                        AppPhase::FileSelected => view! {
                            <div>
                                <FileInfo file=file on_reset=on_reset />
                                <FormatPicker file=file target=target />
                                <button
                                    class="btn btn-primary btn-lg w-full mt-6 gap-2 group"
                                    class:btn-disabled=move || target.get().is_none()
                                    on:click=on_convert
                                >
                                    <span>"Convert"</span>
                                    <svg class="w-5 h-5 group-hover:translate-x-1 transition-transform" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <path d="M5 12h14M12 5l7 7-7 7"/>
                                    </svg>
                                </button>
                            </div>
                        }.into_any(),

                        AppPhase::Converting => view! {
                            <div class="text-center py-12">
                                <span class="loading loading-ring loading-lg text-primary"></span>
                                <p class="mt-4 text-base-content/60 text-sm">"Processing your file with WASM..."</p>
                                <progress class="progress progress-primary w-full mt-4"></progress>
                            </div>
                        }.into_any(),

                        AppPhase::Done { .. } => view! {
                            <div class="text-center py-8">
                                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-success/10 mb-4">
                                    <svg class="w-8 h-8 text-success" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                                        <polyline points="22 4 12 14.01 9 11.01"/>
                                    </svg>
                                </div>
                                <h3 class="text-xl font-bold mb-2">"Conversion complete!"</h3>
                                <p class="text-base-content/60 text-sm mb-6">"Your file was converted locally. Zero bytes were uploaded."</p>
                                <div class="flex flex-col sm:flex-row gap-3 justify-center">
                                    <button class="btn btn-success btn-lg gap-2" on:click=on_download>
                                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                            <polyline points="7 10 12 15 17 10"/>
                                            <line x1="12" y1="15" x2="12" y2="3"/>
                                        </svg>
                                        "Download"
                                    </button>
                                    <button class="btn btn-ghost btn-lg" on:click=on_reset>
                                        "Convert another file"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),

                        AppPhase::Error(msg) => view! {
                            <div class="text-center py-8">
                                <div class="alert alert-error mb-6">
                                    <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <circle cx="12" cy="12" r="10"/>
                                        <line x1="15" y1="9" x2="9" y2="15"/>
                                        <line x1="9" y1="9" x2="15" y2="15"/>
                                    </svg>
                                    <span>{msg}</span>
                                </div>
                                <button class="btn btn-ghost" on:click=on_reset>"Try again"</button>
                            </div>
                        }.into_any(),
                    }
                }}
            </div>
        </section>
    }
}

// ── Drop Zone ───────────────────────────────────────

#[component]
fn DropZone(
    dragging: RwSignal<bool>,
    on_file_selected: impl Fn(String, Vec<u8>) + 'static + Clone,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let cb = on_file_selected.clone();

    let handle_files = move |file: web_sys::File| {
        let cb = cb.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let name = file.name();
            let Ok(buf) = JsFuture::from(file.array_buffer()).await else { return };
            let array = js_sys::Uint8Array::new(&buf);
            let bytes = array.to_vec();
            cb(name, bytes);
        });
    };

    let handle_files_clone = handle_files.clone();

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        dragging.set(false);
        if let Some(dt) = ev.data_transfer() {
            if let Some(files) = dt.files() {
                if let Some(file) = files.get(0) {
                    handle_files_clone(file);
                }
            }
        }
    };

    let on_change = move |_ev: leptos::ev::Event| {
        if let Some(input) = input_ref.get() {
            let el: &web_sys::HtmlInputElement = &input;
            if let Some(files) = el.files() {
                if let Some(file) = files.get(0) {
                    handle_files(file);
                }
            }
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
                if dragging.get() {
                    "relative rounded-xl border-2 border-dashed p-12 sm:p-16 text-center transition-all duration-300 cursor-pointer border-primary bg-primary/10 scale-105"
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
                class="hidden"
                on:change=on_change
            />

            <div class="flex flex-col items-center gap-4">
                <div class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center">
                    <svg class="w-8 h-8 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="17 8 12 3 7 8"/>
                        <line x1="12" y1="3" x2="12" y2="15"/>
                    </svg>
                </div>
                <div>
                    <p class="text-lg font-medium">"Drop your file here"</p>
                    <p class="text-sm text-base-content/50 mt-1">
                        "or "
                        <span class="text-primary underline underline-offset-2">"browse files"</span>
                    </p>
                </div>
                <p class="text-xs text-base-content/30">"Images · Documents · Data · Config files"</p>
            </div>
        </div>
    }
}

// ── File Info ───────────────────────────────────────

#[component]
fn FileInfo(
    file: RwSignal<Option<FileData>>,
    on_reset: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 p-4 rounded-xl bg-base-100/50 border border-white/5">
            <div class="text-3xl">{move || format_icon(&file.get().map(|f| f.extension.clone()).unwrap_or_default())}</div>
            <div class="flex-1 min-w-0">
                <p class="font-medium truncate">{move || file.get().map(|f| f.name.clone()).unwrap_or_default()}</p>
                <p class="text-sm text-base-content/50">
                    {move || file.get().map(|f| format_size(f.size)).unwrap_or_default()}
                    " · "
                    <span class="badge badge-sm badge-primary badge-outline">
                        {move || file.get().map(|f| f.extension.to_uppercase()).unwrap_or_default()}
                    </span>
                </p>
            </div>
            <button class="btn btn-ghost btn-sm btn-circle" on:click=on_reset>
                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
            </button>
        </div>
    }
}

// ── Format Picker ───────────────────────────────────

#[component]
fn FormatPicker(
    file: RwSignal<Option<FileData>>,
    target: RwSignal<Option<String>>,
) -> impl IntoView {
    let formats = Memo::new(move |_| {
        file.get()
            .map(|f| converter::get_output_formats(&f.extension)
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>())
            .unwrap_or_default()
    });

    view! {
        <div class="mt-5">
            <label class="text-sm font-medium text-base-content/60 mb-3 block">"Convert to"</label>
            <div class="flex flex-wrap gap-2">
                <For
                    each=move || formats.get()
                    key=|fmt| fmt.clone()
                    let:fmt
                >
                    {
                        let fmt_click = fmt.clone();
                        let fmt_display = fmt.clone();
                        let fmt_check = fmt.clone();
                        view! {
                            <button
                                class="btn btn-sm rounded-full gap-1 transition-all duration-200"
                                class:btn-primary=move || target.get().as_deref() == Some(&fmt_check)
                                class:btn-outline=move || target.get().as_deref() != Some(&fmt_click)
                                on:click={
                                    let f = fmt.clone();
                                    move |_| target.set(Some(f.clone()))
                                }
                            >
                                {fmt_display.to_uppercase()}
                            </button>
                        }
                    }
                </For>
            </div>
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
                    title="Drop your file"
                    description="Your file is read into browser memory via the File API. It never touches any server."
                />
                <StepCard
                    number="02"
                    icon="⚡"
                    title="WASM converts it"
                    description="Our Rust-compiled WebAssembly engine processes your file at near-native speed, entirely in your browser."
                />
                <StepCard
                    number="03"
                    icon="📥"
                    title="Download the result"
                    description="The converted file is generated locally and downloaded. Verify it yourself — open DevTools and watch the Network tab."
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
fn FormatCard(
    icon: &'static str,
    title: &'static str,
    formats: &'static str,
) -> impl IntoView {
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
                    href="https://github.com/RephlexZero/transfigureing"
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

fn download_blob(data: &[u8], original_name: &str, target_ext: &str) {
    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array.buffer());

    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime_type_for(target_ext));

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let a = document.create_element("a").unwrap();
    a.set_attribute("href", &url).unwrap();

    // Build output filename: input.md -> input.html
    let output_name = if let Some(stem) = original_name.rsplit_once('.') {
        format!("{}.{}", stem.0, target_ext)
    } else {
        format!("{original_name}.{target_ext}")
    };
    a.set_attribute("download", &output_name).unwrap();

    let el: &web_sys::HtmlElement = a.unchecked_ref();
    el.click();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

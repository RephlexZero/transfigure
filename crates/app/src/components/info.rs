use leptos::prelude::*;

// ── How It Works ────────────────────────────────────

#[component]
pub fn HowItWorks() -> impl IntoView {
    view! {
        <section id="how" class="py-14 max-w-6xl mx-auto px-4">
            <h2 class="text-3xl sm:text-4xl font-black uppercase text-center mb-10">
                "How it works "
                <span class="gradient-text">"and what it supports"</span>
            </h2>

            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
                <div class="lg:col-span-5 panel-shell">
                    <h3 class="text-lg font-black uppercase tracking-wide mb-4">"Conversion flow"</h3>
                    <div class="space-y-3">
                        <FlowRow
                            title="Load"
                            description="Drop files or browse from disk."
                            detail="Local memory"
                        />
                        <FlowRow
                            title="Process"
                            description="Rust + WASM converts on your CPU."
                            detail="No server calls"
                        />
                        <FlowRow
                            title="Export"
                            description="Save outputs individually or as an archive."
                            detail="ZIP, TAR.GZ, TAR.XZ, 7Z"
                        />
                    </div>

                    <div class="mt-5 pt-4 border-t border-white/10">
                        <h4 class="text-sm font-black uppercase tracking-wider text-base-content/80 mb-3">"Privacy by architecture"</h4>
                        <p class="text-sm text-base-content/65 leading-relaxed mb-3">
                            "After the app loads, conversion runs locally. You can verify this in the browser network tab during conversion."
                        </p>
                        <div class="flex flex-wrap gap-2">
                            <span class="brutal-chip text-primary border-primary/30 bg-primary/10">"No uploads"</span>
                            <span class="brutal-chip text-secondary border-secondary/30 bg-secondary/10">"No tracking"</span>
                            <span class="brutal-chip text-accent border-accent/30 bg-accent/10">"Works offline"</span>
                        </div>
                    </div>
                </div>

                <div class="lg:col-span-7 panel-shell p-0 divide-y divide-white/10">
                    <FormatCard label="IMG" title="Images" formats="PNG, JPG, WebP, GIF, BMP, TIFF, AVIF, QOI, TGA, HDR, DDS, EXR, ICO"/>
                    <FormatCard label="DOC" title="Documents" formats="Markdown <-> HTML, Markdown -> Text, DOCX -> Text/HTML, RTF -> Text"/>
                    <FormatCard label="DAT" title="Data" formats="CSV <-> JSON, CSV <-> TSV"/>
                    <FormatCard label="CFG" title="Config" formats="JSON <-> YAML, JSON <-> TOML"/>
                    <FormatCard label="ENC" title="Encoding" formats="Base64 <-> Binary"/>
                    <FormatCard label="VEC" title="Vector" formats="SVG -> PNG"/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FlowRow(title: &'static str, description: &'static str, detail: &'static str) -> impl IntoView {
    view! {
        <div class="panel-row grid grid-cols-[100px_minmax(0,1fr)] gap-3 hover:border-primary/30 transition-colors">
            <div>
                <h3 class="text-sm font-black uppercase tracking-wide">{title}</h3>
                <p class="text-xs uppercase tracking-wider text-base-content/45 mt-1">{detail}</p>
            </div>
            <div class="pt-0.5">
                <p class="text-sm text-base-content/65 leading-relaxed">{description}</p>
            </div>
        </div>
    }
}

#[component]
pub fn Formats() -> impl IntoView {
    view! {
        <div class="hidden"></div>
    }
}

#[component]
fn FormatCard(label: &'static str, title: &'static str, formats: &'static str) -> impl IntoView {
    view! {
        <div class="panel-row grid grid-cols-[56px_140px_minmax(0,1fr)] gap-4 items-start hover:bg-white/[0.02] transition-colors border-0">
            <span class="text-xs font-black text-accent/70 uppercase tracking-widest border border-accent/30 px-2 py-1 text-center">{label}</span>
            <h3 class="font-bold text-sm uppercase pt-1">{title}</h3>
            <p class="text-xs text-base-content/55 font-medium leading-relaxed pt-1">{formats}</p>
        </div>
    }
}

#[component]
pub fn PrivacyBanner() -> impl IntoView {
    view! {
        <div class="hidden"></div>
    }
}

// ── Support Banner ─────────────────────────────────

#[component]
pub fn SupportBanner() -> impl IntoView {
    view! {
        <section class="py-8 max-w-2xl mx-auto text-center px-4">
            <div class="glass-card rounded-xl p-8 sm:p-10 border-white/5">
                <h2 class="text-xl sm:text-2xl font-black uppercase tracking-tight mb-3">
                    "Support the project"
                </h2>
                <p class="text-sm text-base-content/60 leading-relaxed mb-6 max-w-md mx-auto font-medium">
                    "Transfigure is entirely open-source and free to use. If it has saved you some time or helped your workflow, consider supporting its development."
                </p>
                <div class="flex flex-wrap justify-center gap-3">
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm gap-2 bg-[#FF5E5B] hover:bg-[#e54f4d] border-none text-white font-bold"
                    >
                        "Support on Ko-fi"
                    </a>
                    <a
                        href="https://github.com/sponsors/RephlexZero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-outline gap-2 hover:border-pink-400 hover:text-pink-400 font-bold"
                    >
                        "Sponsor on GitHub"
                    </a>
                </div>
            </div>
        </section>
    }
}

// ── Footer ──────────────────────────────────────────

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-white/5 py-8">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-base-content/40">
                <div class="flex items-center gap-2">
                    <span class="font-medium uppercase tracking-wider">"Transfigure"</span>
                </div>
                <p>"Built with Rust and WebAssembly. Your files stay on your device."</p>
                <div class="flex items-center gap-4">
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-base-content transition-colors"
                    >"Ko-fi"</a>
                    <a
                        href="https://github.com/sponsors/RephlexZero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-base-content transition-colors"
                    >"Sponsors"</a>
                    <a
                        href="https://github.com/RephlexZero/transfigure"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-base-content transition-colors"
                    >"GitHub"</a>
                </div>
            </div>
        </footer>
    }
}

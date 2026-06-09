use leptos::prelude::*;

// ── How It Works ────────────────────────────────────

#[component]
pub fn HowItWorks() -> impl IntoView {
    view! {
        <section id="how" class="py-14 max-w-6xl mx-auto px-4">
            <div class="flex items-baseline gap-4 mb-8">
                <span class="section-tag">"§ 02 · Method"</span>
                <div class="flex-1 border-b hairline"></div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
                <div class="lg:col-span-5 plate">
                    <div class="space-y-0 divide-y divide-base-content/10">
                        <FlowRow
                            index="01"
                            title="Load"
                            description="Drop files or browse from disk. Bytes are read straight into browser memory."
                        />
                        <FlowRow
                            index="02"
                            title="Transmute"
                            description="The Rust engine decodes, converts and re-encodes on your CPU. No server calls."
                        />
                        <FlowRow
                            index="03"
                            title="Collect"
                            description="Save outputs one by one, or bundle the batch as ZIP, TAR.GZ, TAR.XZ or 7Z."
                        />
                    </div>

                    <div class="mt-6 pt-4 border-t-2 hairline">
                        <h3 class="text-[10px] font-bold uppercase tracking-[0.25em] text-base-content/50 mb-3">
                            "Privacy by architecture"
                        </h3>
                        <p class="text-sm text-base-content/65 leading-relaxed">
                            "This is a structural guarantee, not a policy one. There is no upload "
                            "endpoint to trust — open DevTools during a conversion and watch the "
                            "network stay silent."
                        </p>
                    </div>
                </div>

                <div id="formats" class="lg:col-span-7 plate p-0 sm:p-0 divide-y divide-base-content/10">
                    <FormatCard label="IMG" title="Images" formats="PNG · JPG · WebP · GIF · BMP · TIFF · AVIF · QOI · TGA · HDR · DDS · EXR · ICO"/>
                    <FormatCard label="AUD" title="Audio" formats="MP3 / FLAC / OGG / WAV → WAV"/>
                    <FormatCard label="DOC" title="Documents" formats="Markdown ↔ HTML · MD/HTML/TXT → PDF · PDF → TXT/HTML · DOCX → TXT/HTML · RTF → TXT"/>
                    <FormatCard label="DAT" title="Data" formats="CSV ↔ JSON · CSV ↔ TSV"/>
                    <FormatCard label="CFG" title="Config" formats="JSON ↔ YAML · JSON ↔ TOML"/>
                    <FormatCard label="ENC" title="Encoding" formats="Base64 ↔ Binary"/>
                    <FormatCard label="VEC" title="Vector" formats="SVG → PNG"/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FlowRow(index: &'static str, title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[3rem_minmax(0,1fr)] gap-3 py-4">
            <span class="text-2xl font-black text-primary/40 tabular-nums leading-none">{index}</span>
            <div>
                <h3 class="text-sm font-bold uppercase tracking-[0.2em] mb-1">{title}</h3>
                <p class="text-sm text-base-content/60 leading-relaxed">{description}</p>
            </div>
        </div>
    }
}

#[component]
fn FormatCard(label: &'static str, title: &'static str, formats: &'static str) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[3.5rem_7rem_minmax(0,1fr)] gap-4 items-start px-4 sm:px-6 py-4 hover:bg-base-100/60 transition-colors">
            <span class="ext-chip w-[3.5rem] text-primary/80 border-primary/40">{label}</span>
            <h3 class="font-bold text-xs uppercase tracking-[0.15em] pt-1">{title}</h3>
            <p class="text-xs text-base-content/55 leading-relaxed pt-1">{formats}</p>
        </div>
    }
}

// ── Support Banner ─────────────────────────────────

#[component]
pub fn SupportBanner() -> impl IntoView {
    view! {
        <section class="py-8 max-w-2xl mx-auto text-center px-4">
            <div class="plate">
                <span class="section-tag block mb-4">"§ 03 · Patronage"</span>
                <p class="text-sm text-base-content/60 leading-relaxed mb-6 max-w-md mx-auto">
                    "Transfigure is open-source and free, with no monetisation baked in. "
                    "If it has saved you some time, consider supporting its development."
                </p>
                <div class="flex flex-wrap justify-center gap-3">
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-primary"
                    >
                        "Support on Ko-fi"
                    </a>
                    <a
                        href="https://github.com/sponsors/RephlexZero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-outline border hairline"
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
        <footer class="border-t-2 hairline py-8">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-[11px] uppercase tracking-[0.15em] text-base-content/40">
                <span class="font-bold">"Transfigure"</span>
                <p class="normal-case tracking-normal text-xs">"Built with Rust and WebAssembly. Your files stay on your device."</p>
                <div class="flex items-center gap-5">
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-primary transition-colors"
                    >"Ko-fi"</a>
                    <a
                        href="https://github.com/sponsors/RephlexZero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-primary transition-colors"
                    >"Sponsors"</a>
                    <a
                        href="https://github.com/RephlexZero/transfigure"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="hover:text-primary transition-colors"
                    >"GitHub"</a>
                </div>
            </div>
        </footer>
    }
}

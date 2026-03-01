use leptos::prelude::*;

// ── How It Works ────────────────────────────────────

#[component]
pub fn HowItWorks() -> impl IntoView {
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
                    description="Save files individually or bundle everything into a ZIP, TAR.GZ, TAR.XZ, or 7Z archive. All generated locally."
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
pub fn Formats() -> impl IntoView {
    view! {
        <section id="formats" class="py-20 max-w-5xl mx-auto">
            <h2 class="text-3xl sm:text-4xl font-bold text-center mb-14">
                "Supported "
                <span class="gradient-text">"formats"</span>
            </h2>

            <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                <FormatCard icon="🖼️" title="Images" formats="PNG, JPG, WebP, GIF, BMP, TIFF, AVIF, QOI, TGA, HDR, DDS, EXR, ICO"/>
                <FormatCard icon="✏️" title="Documents" formats="Markdown ↔ HTML, Markdown → Text, DOCX → Text/HTML, RTF → Text"/>
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
pub fn PrivacyBanner() -> impl IntoView {
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

// ── Support Banner ─────────────────────────────────

#[component]
pub fn SupportBanner() -> impl IntoView {
    view! {
        <section class="py-12 max-w-2xl mx-auto text-center">
            <div class="glass-card rounded-2xl p-8 sm:p-10 border-primary/10">
                <p class="text-3xl mb-4">"☕"</p>
                <h2 class="text-xl sm:text-2xl font-bold mb-3">
                    "Free for as long as possible."
                </h2>
                <p class="text-sm text-base-content/50 leading-relaxed mb-6 max-w-md mx-auto">
                    "No limits, no accounts, no ads — and no plans to change that. "
                    "I'm a student in the UK keeping this running on a ~£15/year domain. "
                    "If Transfigure has saved you some time, a small donation is genuinely appreciated."
                </p>
                <div class="flex flex-wrap justify-center gap-3">
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm gap-2 bg-[#FF5E5B] hover:bg-[#e54f4d] border-none text-white"
                    >
                        "☕ Buy me a coffee"
                    </a>
                    <a
                        href="https://github.com/sponsors/RephlexZero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-outline gap-2 hover:border-pink-400 hover:text-pink-400"
                    >
                        "♥ Sponsor on GitHub"
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
                    <span>"⚗️"</span>
                    <span class="font-medium">"Transfigure"</span>
                </div>
                <p>"Built with Rust & WebAssembly. Your files stay yours."</p>
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

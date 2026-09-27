use leptos::prelude::*;

use super::icons::{CategoryTile, GithubMark, Ic, Icon, Logo};
use crate::utils::Category;

#[component]
fn SectionHeading(
    id: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    #[prop(optional)] lede: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="max-w-2xl">
            <p class="eyebrow">{eyebrow}</p>
            <h2 id=id class="section-title mt-2 text-balance">{title}</h2>
            {lede.map(|t| view! { <p class="mt-3 text-muted leading-relaxed text-pretty">{t}</p> })}
        </div>
    }
}

// ── How it works ────────────────────────────────────

#[component]
pub fn HowItWorks() -> impl IntoView {
    view! {
        <section id="how" class="border-t border-line bg-surface/50" aria-labelledby="how-title">
            <div class="mx-auto max-w-5xl px-4 sm:px-6 py-16 sm:py-24">
                <SectionHeading
                    id="how-title"
                    eyebrow="How it works"
                    title="Your files never leave this tab."
                    lede="Transfigure has no backend. The page ships a conversion engine written in Rust and compiled to WebAssembly, and it runs on your own CPU. Privacy comes from how the app is built, not from a policy you have to trust."
                />

                <FlowDiagram/>

                <ol class="mt-12 grid gap-8 sm:grid-cols-3">
                    <Step
                        n="1"
                        title="Add files"
                        text="Drop, choose or paste files. They are read into this tab's memory and go nowhere else."
                    />
                    <Step
                        n="2"
                        title="Convert"
                        text="The engine decodes and re-encodes each file locally, one after another, with no server round trips."
                    />
                    <Step
                        n="3"
                        title="Save"
                        text="Download files one at a time, or save the whole batch as a ZIP, TAR.GZ, TAR.XZ or 7Z archive."
                    />
                </ol>

                <div class="mt-12 flex gap-3 rounded-xl border border-line bg-surface p-4 sm:p-5 text-sm">
                    <Icon icon=Ic::Monitor class="size-5 shrink-0 text-accent-strong mt-px"/>
                    <p class="text-muted leading-relaxed">
                        <span class="font-medium text-fg">"Check it yourself. "</span>
                        "Open your browser's developer tools, switch to the Network tab and convert "
                        "something. No requests are made while files convert."
                    </p>
                </div>
            </div>
        </section>
    }
}

/// File → engine → file, drawn inside a "your device" boundary, with the
/// server that isn't there shown outside it.
#[component]
fn FlowDiagram() -> impl IntoView {
    view! {
        <figure class="mt-10 sm:mt-12" aria-label="Diagram: a file is converted by the engine inside your browser; no server is involved">
            <div class="flex flex-col lg:flex-row lg:items-stretch gap-4">
                <div class="relative flex-1 rounded-2xl border border-dashed border-line-strong p-5 pt-8 sm:p-6 sm:pt-9">
                    <span class="absolute -top-3 left-4 inline-flex items-center gap-1.5 rounded-full border border-line bg-page px-2.5 py-0.5 text-xs font-medium text-muted">
                        <Icon icon=Ic::Monitor class="size-3.5"/>
                        "Your device"
                    </span>
                    <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
                        <FlowNode tile=Category::Image title="holiday.png" meta="4.2 MB"/>
                        <FlowArrow/>
                        <div class="flex-[1.2] flex items-center gap-3 rounded-xl border border-accent/40 bg-accent/[0.06] px-4 py-3.5">
                            <span class="size-9 shrink-0 inline-flex items-center justify-center rounded-lg bg-accent/15 text-accent-strong">
                                <Icon icon=Ic::Cpu class="size-[18px]"/>
                            </span>
                            <div class="min-w-0">
                                <p class="text-sm font-medium">"Transfigure engine"</p>
                                <p class="text-xs text-muted">"WebAssembly, in this tab"</p>
                            </div>
                        </div>
                        <FlowArrow/>
                        <FlowNode tile=Category::Image title="holiday.webp" meta="610 KB"/>
                    </div>
                </div>

                <div class="lg:w-52 flex lg:flex-col items-center justify-center gap-3 rounded-2xl border border-line bg-sunken/60 px-5 py-4 text-center">
                    <span class="relative size-9 inline-flex items-center justify-center rounded-lg bg-fg/[0.05] text-subtle">
                        <Icon icon=Ic::Server class="size-[18px]"/>
                        <span class="absolute h-[1.5px] w-11 rotate-45 bg-bad/70 rounded-full"></span>
                    </span>
                    <div class="text-left lg:text-center">
                        <p class="text-sm font-medium text-muted">"Remote server"</p>
                        <p class="text-xs text-subtle">"Not involved"</p>
                    </div>
                </div>
            </div>
        </figure>
    }
}

#[component]
fn FlowNode(tile: Category, title: &'static str, meta: &'static str) -> impl IntoView {
    view! {
        <div class="flex-1 min-w-0 flex items-center gap-3 rounded-xl border border-line bg-surface px-4 py-3.5 shadow-sm">
            <CategoryTile cat=tile/>
            <div class="min-w-0">
                <p class="font-mono text-[13px] font-medium truncate">{title}</p>
                <p class="text-xs text-subtle tabular-nums">{meta}</p>
            </div>
        </div>
    }
}

#[component]
fn FlowArrow() -> impl IntoView {
    view! {
        <span class="flex justify-center text-subtle shrink-0" aria-hidden="true">
            <Icon icon=Ic::ArrowRight class="size-4 hidden sm:block"/>
            <Icon icon=Ic::ArrowDown class="size-4 sm:hidden"/>
        </span>
    }
}

#[component]
fn Step(n: &'static str, title: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <li>
            <span class="inline-flex size-7 items-center justify-center rounded-full border border-line-strong bg-surface text-[13px] font-semibold tabular-nums">
                {n}
            </span>
            <h3 class="mt-4 font-semibold">{title}</h3>
            <p class="mt-1.5 text-sm text-muted leading-relaxed">{text}</p>
        </li>
    }
}

// ── Formats ─────────────────────────────────────────

#[component]
pub fn Formats() -> impl IntoView {
    view! {
        <section id="formats" class="border-t border-line" aria-labelledby="formats-title">
            <div class="mx-auto max-w-5xl px-4 sm:px-6 py-16 sm:py-24">
                <SectionHeading
                    id="formats-title"
                    eyebrow="Formats"
                    title="What it converts"
                    lede="Drop any of these in and pick an output. Mixed batches are fine: each file gets its own target."
                />

                <div class="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <FormatCard
                        cat=Category::Image
                        title="Images"
                        text="Open iPhone HEIC photos, convert between raster formats, encode AVIF, rasterise SVG, and turn any image into a PDF. Photo orientation is kept."
                        from=&["HEIC", "JPG", "PNG", "WebP", "GIF", "BMP", "TIFF", "ICO", "SVG", "QOI", "TGA", "DDS", "HDR", "EXR"]
                        to=&["JPG", "PNG", "WebP", "AVIF", "PDF", "GIF", "BMP", "TIFF", "ICO", "QOI", "TGA"]
                    />
                    <FormatCard
                        cat=Category::Audio
                        title="Audio"
                        text="Convert songs and recordings between MP3, WAV and FLAC, including Apple M4A (AAC and ALAC). Lossless sources keep their bit depth."
                        from=&["MP3", "M4A", "AAC", "WAV", "FLAC", "OGG", "AIFF", "CAF"]
                        to=&["MP3", "WAV", "FLAC"]
                    />
                    <FormatCard
                        cat=Category::Document
                        title="Documents"
                        text="Word, OpenDocument, RTF, Markdown and HTML with headings, lists, links and tables intact. PDFs are converted by extracting their text."
                        from=&["DOCX", "ODT", "RTF", "PDF", "MD", "HTML", "TXT"]
                        to=&["PDF", "DOCX", "HTML", "MD", "TXT"]
                    />
                    <FormatCard
                        cat=Category::Data
                        title="Spreadsheets"
                        text="Excel and OpenDocument sheets to CSV or JSON, and back to XLSX. Column order and leading zeros survive; semicolon CSVs are detected."
                        from=&["XLSX", "XLS", "ODS", "CSV", "TSV", "JSON"]
                        to=&["XLSX", "CSV", "JSON", "TSV"]
                    />
                    <FormatCard
                        cat=Category::Config
                        title="Structured data"
                        text="Translate configuration and data files between JSON, YAML, TOML and XML, nested structures included."
                        from=&["JSON", "YAML", "TOML", "XML"]
                        to=&["JSON", "YAML", "TOML", "XML"]
                    />
                    <FormatCard
                        cat=Category::Encoding
                        title="Encoding"
                        text="Encode any file as Base64 text, or decode Base64 back to the original file."
                        from=&["Any file", "Base64"]
                        to=&["Base64", "Original file"]
                    />
                </div>

                <p class="mt-6 text-sm text-subtle">
                    "Converted batches can be saved as ZIP, TAR.GZ, TAR.XZ or 7Z archives."
                </p>
            </div>
        </section>
    }
}

#[component]
fn FormatCard(
    cat: Category,
    title: &'static str,
    text: &'static str,
    from: &'static [&'static str],
    to: &'static [&'static str],
) -> impl IntoView {
    let chips = |list: &'static [&'static str]| {
        list.iter()
            .map(|f| view! { <li class="fmt normal-case">{*f}</li> })
            .collect::<Vec<_>>()
    };
    view! {
        <article class="rounded-2xl border border-line bg-surface p-5 flex flex-col">
            <div class="flex items-center gap-3">
                <CategoryTile cat=cat/>
                <h3 class="font-semibold">{title}</h3>
            </div>
            <p class="mt-3 text-sm text-muted leading-relaxed">{text}</p>
            <dl class="mt-4 pt-4 border-t border-line space-y-3 text-xs">
                <div class="grid grid-cols-[2.25rem_minmax(0,1fr)] gap-2 items-start">
                    <dt class="text-subtle pt-1">"From"</dt>
                    <dd><ul class="flex flex-wrap gap-1">{chips(from)}</ul></dd>
                </div>
                <div class="grid grid-cols-[2.25rem_minmax(0,1fr)] gap-2 items-start">
                    <dt class="text-subtle pt-1">"To"</dt>
                    <dd><ul class="flex flex-wrap gap-1">{chips(to)}</ul></dd>
                </div>
            </dl>
        </article>
    }
}

// ── FAQ ─────────────────────────────────────────────

#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <section id="faq" class="border-t border-line bg-surface/50" aria-labelledby="faq-title">
            <div class="mx-auto max-w-5xl px-4 sm:px-6 py-16 sm:py-24 grid gap-10 lg:grid-cols-[18rem_minmax(0,1fr)]">
                <SectionHeading id="faq-title" eyebrow="FAQ" title="Questions"/>

                <div class="divide-y divide-line border-y border-line">
                    <FaqItem q="Are my files really never uploaded?">
                        "Yes. Transfigure is a set of static files with no server-side code. "
                        "Conversion runs in your browser, so there is nowhere for a file to be "
                        "sent. The Network tab in your browser's developer tools will confirm it."
                    </FaqItem>
                    <FaqItem q="Does it work offline?">
                        "Once the page has loaded, converting needs no network connection. You "
                        "can disconnect and keep working."
                    </FaqItem>
                    <FaqItem q="Is there a file size limit?">
                        "There is no fixed limit. Files are held in your browser's memory while "
                        "they convert, so the practical ceiling is your device's available memory."
                    </FaqItem>
                    <FaqItem q="What happens to my files when I close the tab?">
                        "They're gone. Nothing is written to storage or kept between visits."
                    </FaqItem>
                    <FaqItem q="Can it convert iPhone photos?">
                        "Yes. HEIC and HEIF photos convert to JPG, PNG, WebP or PDF, the right "
                        "way up. Drop a whole camera-roll export at once."
                    </FaqItem>
                    <FaqItem q="What can't it do?">
                        "Video isn't supported. PDFs are converted by extracting their text, so "
                        "page layout and pictures aren't kept, and scanned pages have no text to "
                        "extract. PDF output uses built-in fonts that cover Western European "
                        "languages."
                    </FaqItem>
                    <FaqItem q="Is it free?">
                        "Yes. There are no accounts, usage caps or paid tiers. The source is "
                        "available under the AGPL-3.0 licence."
                    </FaqItem>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FaqItem(q: &'static str, children: Children) -> impl IntoView {
    view! {
        <details class="group py-1">
            <summary class="flex cursor-pointer list-none items-center justify-between gap-4 py-4 font-medium [&::-webkit-details-marker]:hidden rounded-md">
                {q}
                <span class="shrink-0 text-subtle transition-transform duration-200 group-open:rotate-45">
                    <Icon icon=Ic::Plus class="size-4"/>
                </span>
            </summary>
            <p class="pb-5 -mt-1 pr-8 text-muted leading-relaxed text-pretty">{children()}</p>
        </details>
    }
}

// ── Support ─────────────────────────────────────────

#[component]
pub fn SupportBanner() -> impl IntoView {
    view! {
        <section class="border-t border-line" aria-label="Support the project">
            <div class="mx-auto max-w-5xl px-4 sm:px-6 py-16 sm:py-20">
                <div class="card flex flex-col md:flex-row md:items-center gap-6 p-6 sm:p-8">
                    <div class="flex-1">
                        <h2 class="text-lg font-semibold tracking-tight">"Free, with nothing to sell you."</h2>
                        <p class="mt-1.5 text-sm text-muted leading-relaxed max-w-lg">
                            "Transfigure is open source and has no ads or tracking. If it saved you "
                            "some time, you can help keep it going."
                        </p>
                    </div>
                    <div class="flex flex-wrap gap-2.5">
                        <a
                            href="https://ko-fi.com/rephlexzero"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="btn btn-primary"
                        >
                            <Icon icon=Ic::Heart class="size-4"/>
                            "Support on Ko-fi"
                        </a>
                        <a
                            href="https://github.com/sponsors/RephlexZero"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="btn btn-secondary"
                        >
                            <GithubMark class="size-4"/>
                            "GitHub Sponsors"
                        </a>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Footer ──────────────────────────────────────────

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-line">
            <div class="mx-auto max-w-6xl px-4 sm:px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-subtle">
                <div class="flex items-center gap-2.5">
                    <Logo class="size-[18px] text-accent"/>
                    <span class="font-medium text-muted">"Transfigure"</span>
                    <span aria-hidden="true">"·"</span>
                    <span>"Built with Rust and WebAssembly"</span>
                </div>
                <nav class="flex items-center gap-5" aria-label="Footer">
                    <a href="https://github.com/RephlexZero/transfigure" target="_blank" rel="noopener noreferrer" class="hover:text-fg transition-colors">"Source"</a>
                    <a href="https://ko-fi.com/rephlexzero" target="_blank" rel="noopener noreferrer" class="hover:text-fg transition-colors">"Ko-fi"</a>
                    <a href="https://github.com/sponsors/RephlexZero" target="_blank" rel="noopener noreferrer" class="hover:text-fg transition-colors">"Sponsors"</a>
                    <span>"AGPL-3.0"</span>
                </nav>
            </div>
        </footer>
    }
}

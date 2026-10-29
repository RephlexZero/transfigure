use leptos::prelude::*;

use super::icons::{GithubMark, Logo};
use crate::state::Batch;
use crate::utils::{all_outputs, category_of, format_counts};

/// Every input format, one per cell, grouped by family.
const GRID: [&str; 39] = [
    "heic", "jpg", "png", "webp", "gif", "bmp", "tiff", "ico", "svg", "qoi", "tga", "dds", "hdr",
    "exr", "mp3", "m4a", "aac", "wav", "flac", "ogg", "aiff", "caf", "docx", "odt", "rtf", "pdf",
    "md", "html", "txt", "xlsx", "xls", "ods", "csv", "tsv", "json", "yaml", "toml", "xml",
    "base64",
];

// ── Format grid ─────────────────────────────────────

/// All the formats it reads, set like a type specimen's character set.
/// Formats in the current queue are filled in source ink.
#[component]
pub fn FormatGrid() -> impl IntoView {
    let batch = expect_context::<Batch>();
    let outputs = all_outputs();
    let (inputs, output_count) = format_counts();

    view! {
        <section aria-labelledby="grid-title" class="pt-2">
            <div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-1 min-h-10">
                <h2 id="grid-title" class="font-mono text-xs font-semibold">
                    {format!("Every format it reads · {inputs} in, {output_count} out")}
                </h2>
                <p class="flex items-center gap-5 label">
                    <span class="flex items-center gap-2">
                        <span class="size-[7px] rounded-full bg-res" aria-hidden="true"></span>
                        "also an output"
                    </span>
                    <span class="flex items-center gap-2">
                        <span class="size-2.5 bg-src" aria-hidden="true"></span>
                        "in your queue"
                    </span>
                </p>
            </div>
            <ul class="grid grid-cols-[repeat(auto-fill,minmax(5.5rem,1fr))] border-t border-l border-rule">
                {GRID
                    .iter()
                    .map(|&ext| {
                        let is_out = outputs.contains(&ext);
                        let queued = Memo::new(move |_| batch.has_ext(ext));
                        let cell = move || {
                            let tone = if queued.get() { "bg-src text-on-src" } else { "" };
                            format!(
                                "relative flex items-center justify-center h-[4.75rem] border-r border-b border-rule transition-colors duration-300 {tone}"
                            )
                        };
                        let code = move || {
                            let tone = if queued.get() { "text-on-src" } else { "text-ink-2" };
                            format!("absolute top-1.5 left-2 font-mono text-[10px] {tone}")
                        };
                        let dot = move || {
                            let tone = if queued.get() { "bg-on-src" } else { "bg-res" };
                            format!("absolute top-2.5 right-2.5 size-[7px] rounded-full {tone}")
                        };
                        let note = move || {
                            let out = if is_out { ", also an output" } else { "" };
                            let mine = if queued.get() { ", in your queue" } else { "" };
                            format!("{out}{mine}")
                        };
                        view! {
                            <li class=cell>
                                <span class=code aria-hidden="true">{category_of(ext).code()}</span>
                                {is_out.then(|| view! { <span class=dot aria-hidden="true"></span> })}
                                <span class="text-[22px] font-semibold tracking-[-0.02em]">{ext}</span>
                                <span class="sr-only">{note}</span>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </section>
    }
}

// ── How it works ────────────────────────────────────

#[component]
pub fn HowItWorks() -> impl IntoView {
    view! {
        <section id="how" aria-labelledby="how-title" class="mt-24 sm:mt-32 border-t-2 border-ink pt-7">
            <div class="flex flex-wrap items-end justify-between gap-6">
                <h2 id="how-title" class="section-title flex-[999_1_32rem]">
                    "Converted in your browser. "
                    <span class="text-res">"Uploaded nowhere."</span>
                </h2>
                <p class="flex-[1_1_18rem] max-w-md text-lg leading-relaxed text-ink-2 text-pretty">
                    "Transfigure has no backend. The page ships a conversion engine written in Rust "
                    "and compiled to WebAssembly, and it runs on your own CPU. Privacy comes from how "
                    "it’s built, not from a policy you have to trust."
                </p>
            </div>

            <ol class="mt-10 grid gap-3 sm:grid-cols-2 lg:grid-cols-4" aria-label="How a file is converted">
                <li class="flex flex-col justify-between gap-6 min-h-[12.5rem] p-5 rounded border-[1.5px] border-ink">
                    <span class="label">"01 · your file"</span>
                    <div>
                        <p class="text-[28px] font-bold leading-tight tracking-[-0.03em]">
                            "holiday"<span class="text-src">".png"</span>
                        </p>
                        <p class="mt-1 text-sm text-ink-2">"Read into this tab’s memory and nowhere else."</p>
                    </div>
                </li>
                <li class="flex flex-col justify-between gap-6 min-h-[12.5rem] p-5 rounded halftone">
                    <span class="self-start rounded bg-paper px-1.5 py-0.5 label">"02 · the engine, in this tab"</span>
                    <div class="self-start rounded bg-paper px-2 py-1.5">
                        <p class="text-[22px] font-bold leading-tight tracking-[-0.02em]">"Rust → WebAssembly"</p>
                        <p class="mt-1 text-sm text-ink-2">"Decodes and re-encodes on your CPU."</p>
                    </div>
                </li>
                <li class="flex flex-col justify-between gap-6 min-h-[12.5rem] p-5 rounded bg-res text-on-res">
                    <span class="font-mono text-xs">"03 · the result"</span>
                    <div>
                        <p class="text-[28px] font-bold leading-tight tracking-[-0.03em]">"holiday.webp"</p>
                        <p class="mt-1 text-sm">"Saved straight to your device."</p>
                    </div>
                </li>
                <li class="flex flex-col justify-between gap-6 min-h-[12.5rem] p-5 rounded border-[1.5px] border-dashed border-rule text-ink-2">
                    <span class="label">"04 · upload server"</span>
                    <div>
                        <p class="text-[28px] font-bold leading-tight tracking-[-0.03em] line-through decoration-res decoration-[3px]">
                            "none"
                        </p>
                        <p class="mt-1 text-sm">"There isn’t one to send files to."</p>
                    </div>
                </li>
            </ol>

            <p class="mt-5 label text-pretty">
                "Check it yourself: open your browser’s developer tools, watch the Network tab and "
                "convert something. No requests are made while files convert."
            </p>
        </section>
    }
}

// ── Formats index ───────────────────────────────────

struct Family {
    name: &'static str,
    note: &'static str,
    reads: &'static str,
    writes: &'static str,
}

const FAMILIES: [Family; 6] = [
    Family {
        name: "Images",
        note: "iPhone HEIC photos, SVG, AVIF output, and any image to PDF. Photos stay the right way up.",
        reads: "heic jpg png webp gif bmp tiff ico svg qoi tga dds hdr exr",
        writes: "jpg png webp avif pdf gif bmp tiff ico qoi tga",
    },
    Family {
        name: "Audio",
        note: "Including Apple M4A (AAC and ALAC). Lossless sources keep their bit depth.",
        reads: "mp3 m4a aac wav flac ogg aiff caf",
        writes: "mp3 wav flac",
    },
    Family {
        name: "Documents",
        note: "Headings, lists, links and tables survive. PDFs are read by extracting their text.",
        reads: "docx odt rtf pdf md html txt",
        writes: "pdf docx html md txt",
    },
    Family {
        name: "Spreadsheets",
        note: "Column order and leading zeros survive; semicolon CSVs are detected.",
        reads: "xlsx xls ods csv tsv json",
        writes: "xlsx csv json tsv",
    },
    Family {
        name: "Data",
        note: "Configuration and data files, nested structures included.",
        reads: "json yaml toml xml",
        writes: "json yaml toml xml",
    },
    Family {
        name: "Encoding",
        note: "Wrap any file as Base64 text, or unwrap it again.",
        reads: "any file, base64",
        writes: "base64, the original file",
    },
];

#[component]
pub fn Formats() -> impl IntoView {
    view! {
        <section id="formats" aria-labelledby="formats-title" class="mt-24 sm:mt-32 border-t-2 border-ink pt-7">
            <div class="flex flex-wrap items-end justify-between gap-4">
                <h2 id="formats-title" class="section-title">"Formats"</h2>
                <p class="label">"mixed batches are fine · save them as .zip .tar.gz .tar.xz .7z"</p>
            </div>
            <div class="mt-8 border-t border-ink md:border-t-0">
                <div class="hidden md:grid grid-cols-[38%_minmax(0,1fr)_minmax(0,1fr)] gap-6 py-2 border-b border-ink label" aria-hidden="true">
                    <span>"Family"</span>
                    <span>"Reads"</span>
                    <span>"Writes"</span>
                </div>
                <ul>
                    {FAMILIES
                        .iter()
                        .map(|f| view! {
                            <li class="grid md:grid-cols-[38%_minmax(0,1fr)_minmax(0,1fr)] gap-x-6 gap-y-2 py-5 border-b border-rule">
                                <div class="mb-1 md:mb-0">
                                    <h3 class="text-[32px] font-bold leading-none tracking-[-0.035em]">{f.name}</h3>
                                    <p class="mt-2 max-w-sm text-sm leading-relaxed text-ink-2 text-pretty">{f.note}</p>
                                </div>
                                <p class="font-mono text-[13px] leading-relaxed text-src-ink">
                                    <span class="md:sr-only inline-block w-16 text-ink-2">"reads"</span>
                                    {f.reads}
                                </p>
                                <p class="font-mono text-[13px] font-semibold leading-relaxed text-res-ink">
                                    <span class="md:sr-only inline-block w-16 font-normal text-ink-2">"writes"</span>
                                    {f.writes}
                                </p>
                            </li>
                        })
                        .collect::<Vec<_>>()}
                </ul>
            </div>
        </section>
    }
}

// ── Questions ───────────────────────────────────────

const QUESTIONS: [(&str, &str); 7] = [
    (
        "Are my files really never uploaded?",
        "Yes. Transfigure is a set of static files with no server-side code. Conversion runs in your browser, so there is nowhere for a file to be sent.",
    ),
    (
        "Does it work offline?",
        "Once the page has loaded, converting needs no network connection. You can disconnect and keep working.",
    ),
    (
        "Is there a file size limit?",
        "No fixed limit. Files are held in your browser’s memory while they convert, so the ceiling is your device’s memory.",
    ),
    (
        "What happens when I close the tab?",
        "Your files are gone. Nothing is written to storage or kept between visits.",
    ),
    (
        "Can it convert iPhone photos?",
        "Yes. HEIC and HEIF photos convert to JPG, PNG, WebP or PDF, the right way up. Drop a whole camera-roll export at once.",
    ),
    (
        "What can’t it do?",
        "Video isn’t supported. PDFs are converted by extracting their text, so layout and pictures aren’t kept, and scanned pages have no text to extract.",
    ),
    (
        "Is it free?",
        "Yes. No accounts, usage caps or paid tiers. The source is available under the AGPL-3.0 licence.",
    ),
];

#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <section id="faq" aria-labelledby="faq-title" class="mt-24 sm:mt-32 border-t-2 border-ink pt-7">
            <h2 id="faq-title" class="section-title">"Questions"</h2>
            <ol class="mt-10 grid gap-x-12 gap-y-8 md:grid-cols-2 xl:grid-cols-3">
                {QUESTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, (q, a))| view! {
                        <li class="grid grid-cols-[2.25rem_minmax(0,1fr)]">
                            <span class="pt-1 font-mono text-[13px] font-semibold text-res-ink tabular-nums" aria-hidden="true">
                                {format!("{:02}", i + 1)}
                            </span>
                            <div>
                                <h3 class="text-xl font-bold leading-snug tracking-[-0.015em] text-balance">{*q}</h3>
                                <p class="mt-1.5 leading-relaxed text-ink-2 text-pretty">{*a}</p>
                            </div>
                        </li>
                    })
                    .collect::<Vec<_>>()}
            </ol>
        </section>
    }
}

// ── Support ─────────────────────────────────────────

#[component]
pub fn SupportBanner() -> impl IntoView {
    view! {
        <section
            aria-label="Support the project"
            class="mt-24 sm:mt-32 py-10 border-y-2 border-ink flex flex-wrap items-center justify-between gap-6"
        >
            <div>
                <p class="text-[clamp(2.25rem,4.4vw,3.75rem)] font-extrabold leading-[0.98] tracking-[-0.04em]">
                    "Free, with nothing to sell you."
                </p>
                <p class="mt-3 max-w-xl text-ink-2 text-pretty">
                    "Transfigure is open source with no ads or tracking. If it saved you some time, "
                    "you can help keep it going."
                </p>
            </div>
            <div class="flex flex-wrap gap-3">
                <a
                    href="https://ko-fi.com/rephlexzero"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn btn-res btn-lg"
                >
                    "Support on Ko-fi"
                </a>
                <a
                    href="https://github.com/sponsors/RephlexZero"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn btn-line btn-lg"
                >
                    <GithubMark class="size-[18px]"/>
                    "GitHub Sponsors"
                </a>
            </div>
        </section>
    }
}

// ── Footer ──────────────────────────────────────────

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="py-8 flex flex-col sm:flex-row sm:items-center justify-between gap-4 label">
            <p class="flex items-center gap-2.5">
                <Logo class="size-5"/>
                <span class="text-ink font-semibold">"Transfigure"</span>
                <span aria-hidden="true">"·"</span>
                <span>"Rust + WebAssembly · AGPL-3.0"</span>
            </p>
            <nav class="flex items-center gap-5" aria-label="Footer">
                <a href="https://github.com/RephlexZero/transfigure" target="_blank" rel="noopener noreferrer" class="hover:text-ink transition-colors">"source"</a>
                <a href="https://ko-fi.com/rephlexzero" target="_blank" rel="noopener noreferrer" class="hover:text-ink transition-colors">"ko-fi"</a>
                <a href="https://github.com/sponsors/RephlexZero" target="_blank" rel="noopener noreferrer" class="hover:text-ink transition-colors">"sponsors"</a>
            </nav>
        </footer>
    }
}

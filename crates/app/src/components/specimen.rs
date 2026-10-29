use leptos::prelude::*;

use super::converter::Picker;
use super::icons::{Ic, Icon};
use crate::state::Batch;
use crate::types::{FileView, Phase};
use crate::utils::{
    Category, category_of, ext_label, ext_scale, format_counts, format_elapsed, format_size,
    format_size_delta, make_output_name,
};

/// Relative sizes of the output formats under the specimen, most useful
/// (and largest) first. Never below 1.25rem, so the chosen one, in result
/// ink, stays large text for contrast.
const PICK_SIZES: [f32; 6] = [1.0, 0.7, 0.54, 0.44, 0.38, 0.34];

fn is_mac() -> bool {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .is_some_and(|ua| ua.contains("Mac"))
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Pending,
    Converting,
    Done,
    Failed,
}

/// Everything the giant extension depends on. A change here remounts it,
/// which is what plays its entrance; anything else (a new target while
/// pending, a size update) leaves it alone.
#[derive(Clone, PartialEq)]
enum Stage {
    Empty,
    File {
        id: usize,
        kind: Kind,
        src: String,
        dst: String,
        /// Shared by every phase so the type doesn't jump in size when a
        /// conversion starts or lands.
        scale_pct: u8,
    },
}

impl Stage {
    fn of(file: Option<&FileView>) -> Self {
        let Some(f) = file else {
            return Stage::Empty;
        };
        let src = ext_label(&f.ext);
        let dst = f.target.as_deref().map(ext_label).unwrap_or_default();
        let scale = ext_scale(&src).min(if dst.is_empty() { 1.0 } else { ext_scale(&dst) });
        let kind = match f.phase {
            Phase::Pending => Kind::Pending,
            Phase::Converting => Kind::Converting,
            Phase::Done { .. } => Kind::Done,
            Phase::Failed(_) => Kind::Failed,
        };
        Stage::File {
            id: f.id,
            kind,
            src,
            // Only the phases that print the result ink depend on it.
            dst: if matches!(kind, Kind::Converting | Kind::Done) {
                dst
            } else {
                String::new()
            },
            scale_pct: (scale * 100.0).round() as u8,
        }
    }
}

/// Cap height, x-height and baseline drawn through the specimen.
#[component]
fn Guides() -> impl IntoView {
    view! {
        <span class="guides">
            <span class="guide at-cap"></span>
            <span class="guide at-x"></span>
            <span class="guide at-base"></span>
            <span class="guide-at at-cap hidden sm:block"><span class="guide-label">"cap height"</span></span>
            <span class="guide-at at-x hidden sm:block"><span class="guide-label">"x-height"</span></span>
            <span class="guide-at at-base hidden sm:block"><span class="guide-label">"baseline"</span></span>
        </span>
    }
}

const LAYER: &str = "col-start-1 row-start-1 mix-blend-multiply dark:mix-blend-screen";

fn render_stage(stage: Stage, picker: Picker) -> AnyView {
    match stage {
        Stage::Empty => view! {
            <button
                type="button"
                class="group block w-full text-left"
                aria-label="Choose files to convert"
                on:click=move |_| picker.open()
            >
                <div class="specimen-stage" aria-hidden="true">
                    <div class="specimen-type">
                        <span class="inline-grid align-baseline">
                            <span class="col-start-1 row-start-1 specimen-dots opacity-0 transition-opacity duration-300 group-hover:opacity-100 group-focus-visible:opacity-100">
                                ".drop"
                            </span>
                            <span class="col-start-1 row-start-1 specimen-outline">".drop"</span>
                        </span>
                        <Guides/>
                    </div>
                </div>
            </button>
        }
        .into_any(),
        Stage::File {
            kind,
            src,
            dst,
            scale_pct,
            ..
        } => {
            let layers = match kind {
                Kind::Pending => view! {
                    <span class=format!("{LAYER} text-src animate-settle")>{src}</span>
                }
                .into_any(),
                Kind::Converting => view! {
                    <span class=format!("{LAYER} text-src")>{src}</span>
                    <span class=format!("{LAYER} text-res animate-overprint motion-reduce:hidden")>{dst}</span>
                }
                .into_any(),
                Kind::Done => view! {
                    <span class=format!("{LAYER} text-src animate-lift")>{src}</span>
                    <span class=format!("{LAYER} text-res animate-register")>{dst}</span>
                }
                .into_any(),
                Kind::Failed => view! {
                    <span class="col-start-1 row-start-1 text-ink-2 line-through decoration-[0.025em] animate-settle">
                        {src}
                    </span>
                }
                .into_any(),
            };
            view! {
                <div class="specimen-stage" aria-hidden="true">
                    <div class="specimen-type" style=format!("--ext-scale:{}", f32::from(scale_pct) / 100.0)>
                        <span class="inline-grid align-baseline isolate">{layers}</span>
                        <Guides/>
                    </div>
                </div>
            }
            .into_any()
        }
    }
}

/// The specimen: the current file's extension set huge, with what it can
/// become underneath.
#[component]
pub fn Specimen() -> impl IntoView {
    let batch = expect_context::<Batch>();
    let picker = expect_context::<Picker>();
    let empty = Memo::new(move |_| batch.is_empty());
    let stage = Memo::new(move |_| batch.shown.with(|f| Stage::of(f.as_ref())));

    view! {
        <section aria-label="Current file" class="min-w-0 pb-6 lg:pr-8">
            <MetaRow/>
            {move || render_stage(stage.get(), picker)}
            <div class="border-t border-rule pt-4">
                <Show when=move || empty.get() fallback=|| view! { <Picks/> }>
                    <Intro/>
                </Show>
            </div>
        </section>
    }
}

/// The ruled line of facts above the specimen.
#[component]
fn MetaRow() -> impl IntoView {
    let batch = expect_context::<Batch>();
    let (inputs, outputs) = format_counts();

    let items = move || {
        let count = batch.stats.with(|s| s.count);
        batch.shown.with(|f| {
            let Some(f) = f else {
                return vec![
                    ("No. —".to_string(), ""),
                    ("waiting for files".to_string(), "text-ink"),
                    (
                        format!("{inputs} formats in · {outputs} out"),
                        "hidden sm:inline",
                    ),
                ];
            };
            let mut items = vec![(format!("No. {:02}", f.number), "")];
            match &f.phase {
                Phase::Done {
                    out_size,
                    elapsed_ms,
                } => {
                    let target = f.target.as_deref().unwrap_or("bin");
                    items.push((make_output_name(&f.name, target), "text-ink font-medium"));
                    items.push((
                        format!("{} → {}", format_size(f.size), format_size(*out_size)),
                        "",
                    ));
                    if let Some(d) = format_size_delta(f.size, *out_size) {
                        items.push((d.replace('-', "\u{2212}"), "text-res-ink"));
                    }
                    items.push((format_elapsed(*elapsed_ms), "hidden sm:inline"));
                }
                _ => {
                    items.push((f.name.clone(), "text-ink font-medium"));
                    items.push((format_size(f.size), ""));
                    items.push((category_of(&f.ext).name().to_string(), "hidden sm:inline"));
                }
            }
            if count > 1 {
                items.push((format!("{} of {count}", f.number), ""));
            }
            items
        })
    };

    view! {
        <p class="flex flex-wrap items-center gap-x-6 gap-y-1 min-h-11 py-2 border-b border-rule font-mono text-xs text-ink-2 tabular-nums">
            {move || {
                items()
                    .into_iter()
                    .map(|(text, class)| {
                        view! { <span class=format!("min-w-0 max-w-full truncate {class}")>{text}</span> }
                    })
                    .collect::<Vec<_>>()
            }}
        </p>
    }
}

/// Headline and first action, before any files are added.
#[component]
fn Intro() -> impl IntoView {
    let picker = expect_context::<Picker>();
    let (mod_key, key_label) = if is_mac() {
        ("⌘", "Command")
    } else {
        ("Ctrl", "Control")
    };
    view! {
        <div class="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
            <div class="max-w-2xl">
                <h1 class="text-[clamp(2.1rem,4.2vw,3.25rem)] font-extrabold leading-[1] tracking-[-0.035em] text-balance">
                    "Convert files. "
                    <span class="text-res">"Upload nothing."</span>
                </h1>
                <p class="mt-3 text-[17px] leading-relaxed text-ink-2 text-pretty">
                    "Images, audio, documents and data, converted on your own device by a Rust "
                    "engine compiled to WebAssembly. Drop files anywhere on this page."
                </p>
            </div>
            <div class="flex flex-col gap-3 lg:hidden">
                <button type="button" class="btn btn-ink btn-lg w-full sm:w-auto" on:click=move |_| picker.open()>
                    "Choose files"
                </button>
                <p class="hidden sm:flex items-center gap-1.5 label">
                    "or paste with"
                    <kbd class="kbd" title=key_label>{mod_key}</kbd>
                    <kbd class="kbd">"V"</kbd>
                </p>
            </div>
        </div>
    }
}

/// The formats the shown file can become, largest first; the chosen one is
/// in result ink.
#[component]
fn Picks() -> impl IntoView {
    let batch = expect_context::<Batch>();
    // Rebuild the buttons only for a different file, so choosing a format
    // keeps focus on the button just pressed.
    let key = Memo::new(move |_| {
        batch.shown.with(|f| {
            f.as_ref()
                .map(|f| (f.id, f.name.clone(), f.ext.clone(), f.formats.clone()))
        })
    });
    let target = Memo::new(move |_| {
        batch
            .shown
            .with(|f| f.as_ref().and_then(|f| f.target.clone()))
    });
    let busy = Memo::new(move |_| {
        batch
            .shown
            .with(|f| f.as_ref().is_some_and(FileView::is_busy))
    });
    let failure = Memo::new(move |_| {
        batch.shown.with(|f| match f.as_ref().map(|f| &f.phase) {
            Some(Phase::Failed(e)) => Some(e.clone()),
            _ => None,
        })
    });

    view! {
        <h1 class="sr-only">"Transfigure: convert files in your browser"</h1>
        {move || failure.get().map(|e| view! {
            <p class="mb-3 flex items-start gap-2 text-[15px] animate-settle" role="alert">
                <Icon icon=Ic::Alert class="size-[18px] mt-0.5 shrink-0"/>
                <span>
                    <span class="font-semibold">"Couldn’t convert this file. "</span>
                    <span class="text-ink-2">{e}</span>
                    " Choose another format to try again."
                </span>
            </p>
        })}
        {move || key.get().map(|(id, name, ext, formats)| {
            if formats.is_empty() {
                return view! {
                    <p class="text-lg text-pretty">
                        "Transfigure can’t read this type of file. "
                        <a href="#formats" class="underline decoration-res decoration-2 underline-offset-4">
                            "See what it reads"
                        </a>
                    </p>
                }
                .into_any();
            }
            // Unknown types can still be wrapped as Base64; say why that's all.
            let unknown = (category_of(&ext) == Category::Other).then(|| view! {
                <p class="mb-3 text-[15px] text-ink-2 text-pretty">
                    {format!("Transfigure doesn’t read {} files, so it can only wrap this one as Base64 text.", ext_label(&ext))}
                </p>
            });
            view! {
                {unknown}
                <div role="group" aria-label=format!("Output format for {name}") class="flex flex-wrap items-baseline gap-x-4 sm:gap-x-6">
                    <span class="label mr-1" aria-hidden="true">"becomes"</span>
                    {formats
                        .into_iter()
                        .enumerate()
                        .map(|(i, fmt)| {
                            let factor = PICK_SIZES.get(i).copied().unwrap_or(0.3);
                            view! {
                                <button
                                    type="button"
                                    class="pick"
                                    style=format!("font-size:max(1.25rem, calc(var(--pick) * {factor}))")
                                    aria-pressed=move || (target.get().as_deref() == Some(fmt)).to_string()
                                    prop:disabled=move || busy.get()
                                    on:click=move |_| batch.set_target(id, fmt.to_string())
                                >
                                    {ext_label(fmt)}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            }
            .into_any()
        })}
    }
}

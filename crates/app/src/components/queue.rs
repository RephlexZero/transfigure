use leptos::prelude::*;

use super::converter::Picker;
use super::icons::{Ic, Icon};
use crate::state::{ARCHIVE_FORMATS, Batch};
use crate::types::{FileView, Phase};
use crate::utils::{ext_label, format_size, format_size_delta, make_output_name};

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// What the button at the foot of the queue offers.
#[derive(Clone, PartialEq)]
enum Action {
    Empty,
    /// Files present but nothing to convert or save (all failed or unreadable).
    Idle,
    Ready(usize),
    Converting,
    /// Some files converted. `single` is the output name when there is one.
    Finished {
        single: Option<String>,
    },
}

/// The batch as a numbered list beside the specimen.
#[component]
pub fn Queue() -> impl IntoView {
    let batch = expect_context::<Batch>();
    let picker = expect_context::<Picker>();
    let empty = Memo::new(move |_| batch.is_empty());
    let show_common = Memo::new(move |_| batch.common_formats.with(|c| !c.is_empty()));
    let show_quality = Memo::new(move |_| batch.stats.with(|s| s.lossy_queued));

    let summary = move || {
        batch.stats.with(|s| {
            if s.count == 0 {
                "empty".to_string()
            } else {
                format!(
                    "{} · {}",
                    plural(s.count, "file", "files"),
                    format_size(s.total_size)
                )
            }
        })
    };

    // On phones an empty queue has nothing to say; the specimen carries the
    // first action instead.
    let class = move || {
        let display = if empty.get() {
            "hidden lg:flex"
        } else {
            "flex"
        };
        format!(
            "{display} flex-col min-w-0 border-t border-ink pt-4 pb-6 lg:border-t-0 lg:border-l lg:border-rule lg:pl-8 lg:pt-5"
        )
    };

    view! {
        <section aria-labelledby="queue-title" class=class>
            <div class="flex items-center justify-between gap-3">
                <h2 id="queue-title" class="text-[28px] font-bold tracking-[-0.03em] leading-none">"Queue"</h2>
                <div class="flex items-center gap-1">
                    <span class="label mr-2 tabular-nums">{summary}</span>
                    <Show when=move || !empty.get()>
                        <button type="button" class="btn-text" on:click=move |_| picker.open()>
                            <Icon icon=Ic::Plus class="size-3.5"/>
                            "add"
                        </button>
                        <button
                            type="button"
                            class="btn-text"
                            prop:disabled=move || batch.converting.get()
                            on:click=move |_| batch.clear()
                        >
                            "clear"
                        </button>
                    </Show>
                </div>
            </div>

            <div class="mt-3 border-t border-ink">
                <Show when=move || show_common.get()>
                    <SetAll/>
                </Show>
                <Show when=move || !empty.get() fallback=|| view! { <BlankRows/> }>
                    <ul class="max-h-[min(26rem,55vh)] overflow-y-auto overscroll-contain" aria-label="Files">
                        <For each=move || batch.ids() key=|id| *id let:id>
                            <Row id=id/>
                        </For>
                    </ul>
                </Show>
            </div>

            <Show when=move || show_quality.get()>
                <Quality/>
            </Show>

            <div class="mt-auto pt-6">
                <ActionArea/>
            </div>
        </section>
    }
}

/// "Set all to" for batches whose files share output formats.
#[component]
fn SetAll() -> impl IntoView {
    let batch = expect_context::<Batch>();
    view! {
        <label class="flex items-center justify-between gap-3 py-2 border-b border-rule">
            <span class="label">"set all to"</span>
            <select
                class="select"
                prop:disabled=move || batch.converting.get()
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if !value.is_empty() {
                        batch.set_all_targets(value);
                    }
                }
            >
                <option value="" disabled=true prop:selected=move || batch.uniform_target.get().is_none()>
                    "mixed"
                </option>
                {move || {
                    batch
                        .common_formats
                        .get()
                        .into_iter()
                        .map(|fmt| view! {
                            <option
                                value=fmt
                                prop:selected=move || batch.uniform_target.get().as_deref() == Some(fmt)
                            >
                                {ext_label(fmt)}
                            </option>
                        })
                        .collect::<Vec<_>>()
                }}
            </select>
        </label>
    }
}

/// Numbered blank lines and the first action, while the queue is empty.
#[component]
fn BlankRows() -> impl IntoView {
    let picker = expect_context::<Picker>();
    view! {
        <ol class="font-mono text-[13px] text-ink-2" aria-hidden="true">
            {(1..=4)
                .map(|n| view! {
                    <li class="flex items-center gap-3 h-12 border-b border-rule">
                        <span class="w-7 text-[11px] tabular-nums">{format!("{n:02}")}</span>
                        <span>"—"</span>
                    </li>
                })
                .collect::<Vec<_>>()}
        </ol>
        <button type="button" class="btn btn-ink btn-lg w-full mt-6" on:click=move |_| picker.open()>
            "Choose files"
        </button>
        <p class="mt-3 label text-center">"or drop them anywhere · paste works too"</p>
    }
}

#[component]
fn Quality() -> impl IntoView {
    let batch = expect_context::<Batch>();
    view! {
        <label class="mt-3 flex items-center gap-3 label animate-settle">
            <span>"quality"</span>
            <input
                type="range"
                min="10"
                max="100"
                step="5"
                class="range flex-1"
                aria-label="Quality for JPG, AVIF and MP3 output"
                prop:value=move || batch.quality.get().to_string()
                prop:disabled=move || batch.converting.get()
                on:input=move |ev| {
                    if let Ok(v) = event_target_value(&ev).parse::<u8>() {
                        batch.quality.set(v);
                    }
                }
            />
            <span class="w-7 text-right text-ink tabular-nums">{move || batch.quality.get()}</span>
        </label>
    }
}

/// One file in the queue. Selecting it shows it in the specimen.
#[component]
fn Row(id: usize) -> impl IntoView {
    let batch = expect_context::<Batch>();
    let Some(initial) = batch
        .views
        .with_untracked(|v| v.iter().find(|f| f.id == id).cloned())
    else {
        return ().into_any();
    };
    let FileView { name, formats, .. } = initial;
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), format!(".{ext}")),
        _ => (name.clone(), String::new()),
    };

    let file = Memo::new(move |_| batch.view_of(id));
    let number = Memo::new(move |_| file.with(|f| f.as_ref().map_or(0, |f| f.number)));
    let phase = Memo::new(move |_| file.with(|f| f.as_ref().map(|f| f.phase.clone())));
    let target = Memo::new(move |_| file.with(|f| f.as_ref().and_then(|f| f.target.clone())));
    let busy = move || phase.get() == Some(Phase::Converting);
    let is_done = Memo::new(move |_| matches!(phase.get(), Some(Phase::Done { .. })));
    let input_size = file.with_untracked(|f| f.as_ref().map_or(0, |f| f.size));

    let meta = move || {
        match phase.get() {
        Some(Phase::Converting) => view! {
            <span class="flex items-center gap-2">
                "converting"
                <span class="relative block h-[2px] w-12 overflow-hidden bg-rule" aria-hidden="true">
                    <span class="absolute inset-y-0 left-0 w-1/3 bg-res animate-sweep motion-reduce:animate-none"></span>
                </span>
            </span>
        }
        .into_any(),
        Some(Phase::Done { out_size, .. }) => {
            let delta = format_size_delta(input_size, out_size)
                .map(|d| format!(" {}", d.replace('-', "\u{2212}")))
                .unwrap_or_default();
            view! {
                <span>
                    {format!("{} → ", format_size(input_size))}
                    <span class="text-res-ink">{format!("{}{delta}", format_size(out_size))}</span>
                </span>
            }
            .into_any()
        }
        Some(Phase::Failed(e)) => view! {
            <span class="flex items-center gap-1.5 text-ink" title=e>
                <Icon icon=Ic::Alert class="size-3.5 shrink-0"/>
                "failed"
            </span>
        }
        .into_any(),
        _ => view! { <span>{format_size(input_size)}</span> }.into_any(),
    }
    };

    let output_name = {
        let name = name.clone();
        Memo::new(move |_| make_output_name(&name, target.get().as_deref().unwrap_or("bin")))
    };
    let has_formats = !formats.is_empty();

    view! {
        <li
            class="queue-row"
            aria-current=move || (batch.shown_id() == Some(id)).then_some("true")
        >
            <span class="pl-1 font-mono text-[11px] text-ink-2 tabular-nums" aria-hidden="true">
                {move || format!("{:02}", number.get())}
            </span>

            <button
                type="button"
                class="min-w-0 py-1 text-left rounded"
                title=name.clone()
                on:click=move |_| batch.focus(id)
            >
                <span class="block truncate font-mono text-[13px]">
                    {stem}
                    <span class="text-src-ink">{ext}</span>
                </span>
                <span class="block truncate font-mono text-[11px] text-ink-2 tabular-nums">{meta}</span>
            </button>

            {if has_formats {
                let label = format!("Output format for {name}");
                view! {
                    <select
                        class="select w-[5.75rem]"
                        aria-label=label
                        prop:disabled=busy
                        on:change=move |ev| batch.set_target(id, event_target_value(&ev))
                    >
                        {formats
                            .into_iter()
                            .map(|fmt| view! {
                                <option value=fmt prop:selected=move || target.get().as_deref() == Some(fmt)>
                                    {ext_label(fmt)}
                                </option>
                            })
                            .collect::<Vec<_>>()}
                    </select>
                }
                .into_any()
            } else {
                view! { <span class="label w-[5.75rem]">"unreadable"</span> }.into_any()
            }}

            <Show
                when=move || is_done.get()
                fallback={
                    let name = name.clone();
                    move || view! {
                        <button
                            type="button"
                            class="icon-btn"
                            prop:disabled=busy
                            aria-label=format!("Remove {name}")
                            title="Remove"
                            on:click=move |_| batch.remove(id)
                        >
                            <Icon icon=Ic::X class="size-4"/>
                        </button>
                    }
                }
            >
                <button
                    type="button"
                    class="icon-btn text-res-ink"
                    aria-label=move || format!("Save {}", output_name.get())
                    title="Save"
                    on:click=move |_| batch.save(id)
                >
                    <Icon icon=Ic::Download class="size-4"/>
                </button>
            </Show>
        </li>
    }
    .into_any()
}

/// The main button and a live status line.
#[component]
fn ActionArea() -> impl IntoView {
    let batch = expect_context::<Batch>();

    let action = Memo::new(move |_| {
        let s = batch.stats.get();
        if s.count == 0 {
            Action::Empty
        } else if batch.converting.get() {
            Action::Converting
        } else if s.ready > 0 {
            Action::Ready(s.ready)
        } else if s.done > 0 {
            Action::Finished {
                single: batch
                    .single_done()
                    .map(|f| make_output_name(&f.name, f.target.as_deref().unwrap_or("bin"))),
            }
        } else {
            Action::Idle
        }
    });

    let status = move || {
        let s = batch.stats.get();
        if batch.converting.get() {
            let (done, total) = batch.progress.get();
            return format!("converting {} of {total}", (done + 1).min(total));
        }
        match action.get() {
            Action::Empty => String::new(),
            Action::Ready(n) => {
                format!("{} ready · nothing is uploaded", plural(n, "file", "files"))
            }
            _ => {
                let mut text = format!("{} converted", plural(s.done, "file", "files"));
                if s.done > 0 {
                    text.push_str(&format!(" · {}", format_size(s.output_size)));
                }
                if s.failed > 0 {
                    text.push_str(&format!(" · {} failed", s.failed));
                }
                text
            }
        }
    };

    let percent = move || {
        let (done, total) = batch.progress.get();
        if total == 0 {
            0.0
        } else {
            done as f64 / total as f64
        }
    };

    let start_over = move || {
        view! {
            <button type="button" class="btn-text" on:click=move |_| batch.clear()>
                <Icon icon=Ic::RotateCcw class="size-3.5"/>
                "start over"
            </button>
        }
    };

    view! {
        {move || match action.get() {
            Action::Empty => ().into_any(),
            Action::Ready(n) => view! {
                <button
                    type="button"
                    class="btn btn-res btn-lg w-full justify-between"
                    on:click=move |_| batch.convert_all()
                >
                    {format!("Convert {}", plural(n, "file", "files"))}
                    <Icon icon=Ic::ArrowRight class="size-5"/>
                </button>
            }
            .into_any(),
            Action::Converting => view! {
                <div class="h-14 px-5 flex items-center justify-between gap-4 rounded border-[1.5px] border-ink">
                    <span class="text-lg font-bold tracking-[-0.01em] tabular-nums">
                        {move || {
                            let (done, total) = batch.progress.get();
                            format!("Converting {} of {total}", (done + 1).min(total))
                        }}
                    </span>
                    <span class="label hidden sm:inline">"0 bytes uploaded"</span>
                </div>
                <div class="mt-2 h-[3px] overflow-hidden bg-rule" aria-hidden="true">
                    <div
                        class="h-full bg-res origin-left transition-transform duration-300 ease-settle"
                        style=move || format!("transform:scaleX({})", percent())
                    ></div>
                </div>
            }
            .into_any(),
            Action::Finished { single } => {
                let save = match single {
                    Some(name) => view! {
                        <button
                            type="button"
                            class="btn btn-res btn-lg w-full justify-between min-w-0"
                            on:click=move |_| {
                                if let Some(f) = batch.single_done() {
                                    batch.save(f.id);
                                }
                            }
                        >
                            <span class="truncate">{format!("Save {name}")}</span>
                            <Icon icon=Ic::Download class="size-5 shrink-0"/>
                        </button>
                    }
                    .into_any(),
                    None => view! {
                        <div class="flex gap-2">
                            <select
                                class="select select-plain h-14 pl-3.5 pr-8 text-sm"
                                aria-label="Archive format"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    if let Some((fmt, _)) = ARCHIVE_FORMATS.iter().find(|(f, _)| *f == value) {
                                        batch.archive.set(fmt);
                                    }
                                }
                            >
                                {ARCHIVE_FORMATS
                                    .iter()
                                    .map(|(fmt, label)| view! {
                                        <option value=*fmt prop:selected=move || batch.archive.get() == *fmt>{*label}</option>
                                    })
                                    .collect::<Vec<_>>()}
                            </select>
                            <button
                                type="button"
                                class="btn btn-res btn-lg flex-1 justify-between"
                                on:click=move |_| batch.save_all()
                            >
                                "Save all"
                                <Icon icon=Ic::Download class="size-5"/>
                            </button>
                        </div>
                    }
                    .into_any(),
                };
                view! {
                    {save}
                    <div class="mt-2 flex justify-end">{start_over()}</div>
                }
                .into_any()
            }
            Action::Idle => view! { <div class="flex justify-end">{start_over()}</div> }.into_any(),
        }}
        <p class="mt-3 label tabular-nums min-h-[1rem]" role="status" aria-live="polite">{status}</p>
    }
}

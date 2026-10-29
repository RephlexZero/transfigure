use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

use super::queue::Queue;
use super::specimen::Specimen;
use crate::state::Batch;

/// Opens the hidden file input. Shared through context so any "choose
/// files" control can use it.
#[derive(Clone, Copy)]
pub struct Picker(NodeRef<html::Input>);

impl Picker {
    pub fn open(&self) {
        if let Some(input) = self.0.get_untracked() {
            let el: &web_sys::HtmlInputElement = &input;
            el.click();
        }
    }
}

/// Read every file in a `FileList` and hand them over together once all
/// have loaded, in the order given (reads finish in any order), so a dropped
/// batch lands in one update.
fn read_files(file_list: web_sys::FileList, batch: Batch) {
    let count = file_list.length();
    if count == 0 {
        return;
    }
    let collected = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let remaining = std::rc::Rc::new(std::cell::Cell::new(count));

    for i in 0..count {
        let Some(file) = file_list.get(i) else {
            remaining.set(remaining.get() - 1);
            continue;
        };
        let collected = collected.clone();
        let remaining = remaining.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let name = file.name();
            if let Ok(buf) = JsFuture::from(file.array_buffer()).await {
                let array = js_sys::Uint8Array::new(&buf);
                collected.borrow_mut().push((i, name, array.to_vec()));
            }
            let left = remaining.get() - 1;
            remaining.set(left);
            if left == 0 {
                let mut items: Vec<_> = collected.borrow_mut().drain(..).collect();
                items.sort_by_key(|(i, _, _)| *i);
                batch.add(items.into_iter().map(|(_, n, b)| (n, b)).collect());
            }
        });
    }
}

fn drag_has_files(ev: &web_sys::DragEvent) -> bool {
    ev.data_transfer()
        .is_some_and(|dt| dt.types().includes(&"Files".into(), 0))
}

/// The tool: the specimen of the current file beside the queue, plus the
/// page-wide drop target and paste handling.
#[component]
pub fn ConverterSection() -> impl IntoView {
    let batch = expect_context::<Batch>();
    let input_ref = NodeRef::<html::Input>::new();
    provide_context(Picker(input_ref));

    let on_input_change = move |_ev: ev::Event| {
        if let Some(input) = input_ref.get_untracked() {
            let el: &web_sys::HtmlInputElement = &input;
            if let Some(list) = el.files() {
                read_files(list, batch);
            }
            el.set_value("");
        }
    };

    // Counter rather than bool: dragenter/dragleave fire for every element
    // crossed, and a bool flickers off mid-drag.
    let drag_depth = RwSignal::new(0i32);
    let dragging = Memo::new(move |_| drag_depth.get() > 0);

    let _ = window_event_listener(ev::dragenter, move |ev| {
        if drag_has_files(&ev) {
            ev.prevent_default();
            drag_depth.update(|d| *d += 1);
        }
    });
    let _ = window_event_listener(ev::dragover, move |ev| {
        if drag_has_files(&ev) {
            ev.prevent_default();
        }
    });
    let _ = window_event_listener(ev::dragleave, move |ev| {
        if drag_has_files(&ev) {
            drag_depth.update(|d| *d = (*d - 1).max(0));
        }
    });
    let _ = window_event_listener(ev::drop, move |ev| {
        drag_depth.set(0);
        if let Some(list) = ev.data_transfer().and_then(|dt| dt.files())
            && list.length() > 0
        {
            ev.prevent_default();
            read_files(list, batch);
        }
    });
    let _ = window_event_listener_untyped("paste", move |ev| {
        let ev: web_sys::ClipboardEvent = ev.unchecked_into();
        if let Some(list) = ev.clipboard_data().and_then(|dt| dt.files())
            && list.length() > 0
        {
            ev.prevent_default();
            read_files(list, batch);
        }
    });

    let accept: String = converter::ALL_INPUT_FORMATS
        .iter()
        .map(|ext| format!(".{ext}"))
        .collect::<Vec<_>>()
        .join(",");

    view! {
        <input
            node_ref=input_ref
            type="file"
            multiple=true
            accept=accept
            class="hidden"
            tabindex="-1"
            aria-hidden="true"
            on:change=on_input_change
        />

        <DragOverlay dragging=dragging/>

        <div
            id="converter"
            class="grid lg:grid-cols-[minmax(0,1fr)_minmax(22rem,27rem)] border-b border-ink scroll-mt-6"
        >
            <Specimen/>
            <Queue/>
        </div>
    }
}

/// Shown over the whole page while files are dragged in. The word lands in
/// both inks, the result ink settling just off register over the source.
#[component]
fn DragOverlay(dragging: Memo<bool>) -> impl IntoView {
    view! {
        <Show when=move || dragging.get()>
            <div class="fixed inset-0 z-50 p-3 sm:p-6 bg-paper/95 animate-fade-in pointer-events-none">
                <div class="size-full rounded border-2 border-dashed border-ink halftone-soft flex flex-col items-center justify-center gap-5 px-6 text-center">
                    <p
                        class="isolate grid font-extrabold leading-none tracking-[-0.05em] text-[clamp(5rem,16vw,14rem)]"
                        aria-hidden="true"
                    >
                        <span class="col-start-1 row-start-1 text-src mix-blend-multiply dark:mix-blend-screen">".drop"</span>
                        <span class="col-start-1 row-start-1 text-res mix-blend-multiply dark:mix-blend-screen animate-misregister">
                            ".drop"
                        </span>
                    </p>
                    <p class="bg-paper px-3 py-1.5 rounded">
                        <span class="block text-xl font-bold tracking-[-0.02em]">"Release to add files"</span>
                        <span class="block label mt-1">"they stay on this device"</span>
                    </p>
                </div>
            </div>
        </Show>
    }
}

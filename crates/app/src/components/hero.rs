use leptos::prelude::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="pt-10 pb-8 sm:pt-14 sm:pb-10 h-full">
            <div class="plate h-full flex flex-col">
                <div class="flex items-center justify-between mb-6 pb-3 border-b hairline">
                    <span class="section-tag">"§ 00 · Transfigure"</span>
                    <span class="text-[10px] uppercase tracking-[0.2em] text-base-content/35">
                        "Rust → WASM"
                    </span>
                </div>

                <h1 class="text-4xl sm:text-6xl lg:text-[4.25rem] font-black uppercase leading-[0.95] tracking-tight mb-8">
                    "Convert files."
                    <br/>
                    <span class="text-primary">"Upload nothing."</span>
                </h1>

                <p class="text-base sm:text-lg text-base-content/70 max-w-xl leading-relaxed mb-8">
                    "Every conversion happens inside this page — a Rust engine compiled to "
                    "WebAssembly, running on your machine. There is no server to receive your "
                    "files; you can watch the network tab and see nothing leave."
                </p>

                // Spec sheet: how the instrument is built.
                <div class="mt-auto border-t-2 hairline divide-y divide-base-content/10 text-sm">
                    <SpecRow label="Engine" value="Pure Rust, compiled to WebAssembly"/>
                    <SpecRow label="Network" value="Zero bytes leave the device"/>
                    <SpecRow label="Account" value="None. No limits, no tracking"/>
                    <SpecRow label="Offline" value="Works after first load"/>
                    <SpecRow label="Source" value="Open · AGPL-3.0"/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn SpecRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[6.5rem_minmax(0,1fr)] gap-3 py-2.5">
            <span class="text-[10px] uppercase tracking-[0.25em] text-primary/80 font-bold pt-0.5">{label}</span>
            <span class="text-base-content/75">{value}</span>
        </div>
    }
}

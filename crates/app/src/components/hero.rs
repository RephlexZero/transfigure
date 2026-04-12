use leptos::prelude::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="pt-10 pb-8 sm:pt-14 sm:pb-10 font-mono h-full">
            <div class="panel-shell border-primary/35 shadow-[8px_8px_0px_0px_rgba(167,139,250,0.3)] relative h-full">

                <div class="text-xs text-secondary/80 mb-7 uppercase tracking-widest flex items-center border-b-2 border-primary/20 pb-2">
                    <span class="flex items-center gap-2">
                        <span class="w-2 h-2 bg-accent inline-block"></span>
                        "Private conversion in your browser"
                    </span>
                </div>

                <h1 class="text-4xl sm:text-6xl lg:text-7xl font-black uppercase tracking-tight mb-8 leading-none text-base-content selection:bg-accent selection:text-base-100">
                    "Convert files"
                    <br />
                    <span class="gradient-text">"without sending them anywhere."</span>
                </h1>

                <p class="text-lg sm:text-xl text-base-content/80 max-w-2xl mb-8 leading-relaxed border-l-4 border-accent pl-4 normal-case">
                    "Transfigure runs on your device using Rust and WebAssembly, so conversion happens locally from start to finish."
                    <br />
                    <span class="font-bold text-accent">"No uploads. No account. No file retention."</span>
                </p>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm font-medium pt-2">
                    <div class="panel-row bg-black/20 flex items-start gap-3 hover:border-accent/40 transition-colors">
                        <span class="text-accent mt-0.5">"//"</span>
                        <div class="text-base-content/70 uppercase">
                            <span class="block text-base-content mb-1">"Privacy first"</span>
                            "Processing stays in browser memory."
                        </div>
                    </div>
                    <div class="panel-row bg-black/20 flex items-start gap-3 hover:border-secondary/40 transition-colors">
                        <span class="text-secondary mt-0.5">"//"</span>
                        <div class="text-base-content/70 uppercase">
                            <span class="block text-base-content mb-1">"Fast by default"</span>
                            "Rust + WASM, running on your CPU."
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

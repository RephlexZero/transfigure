use leptos::prelude::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="pt-20 pb-12 sm:pt-28 sm:pb-16 text-center max-w-3xl mx-auto">
            <div class="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-primary/10 border border-primary/20 text-sm text-primary mb-8">
                <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                </span>
                "100% Client-Side · Zero Uploads"
            </div>

            <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold tracking-tight leading-tight mb-6">
                "Convert files."
                <br/>
                <span class="gradient-text">"Without the trust issues."</span>
            </h1>

            <p class="text-lg sm:text-xl text-base-content/60 max-w-2xl mx-auto leading-relaxed">
                "Your files never leave your browser. Everything runs locally via "
                <span class="text-accent font-medium">"WebAssembly"</span>
                " at near-native speed. No servers. No uploads. No BS."
            </p>
        </section>
    }
}

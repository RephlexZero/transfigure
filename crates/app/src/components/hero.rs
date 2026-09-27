use leptos::prelude::*;

use super::icons::{Ic, Icon};

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-2xl text-center">
            <a
                href="#how"
                class="inline-flex items-center gap-2 rounded-full border border-line bg-surface/80 pl-2.5 pr-3 py-1 text-[13px] text-muted shadow-sm hover:text-fg transition-colors"
            >
                <span class="relative flex size-2" aria-hidden="true">
                    <span class="absolute inset-0 rounded-full bg-ok/40 animate-ping [animation-duration:2.5s]"></span>
                    <span class="relative size-2 rounded-full bg-ok"></span>
                </span>
                "Runs entirely in your browser"
            </a>

            <h1 class="mt-6 text-[2.6rem] leading-[1.05] sm:text-6xl sm:leading-[1.02] font-semibold tracking-[-0.028em] text-balance">
                "Convert files."
                <br/>
                <span class="text-subtle">"Upload nothing."</span>
            </h1>

            <p class="mt-5 text-base sm:text-lg text-muted leading-relaxed text-balance max-w-xl mx-auto">
                "Images, audio, documents and data, converted on your own device by a Rust "
                "engine compiled to WebAssembly. No uploads, no accounts, no limits."
            </p>
        </div>
    }
}

/// Three short guarantees under the converter.
#[component]
pub fn TrustRow() -> impl IntoView {
    view! {
        <ul class="mx-auto max-w-3xl mt-6 grid grid-cols-1 sm:grid-cols-3 gap-x-6 gap-y-2.5 text-[13px] text-muted">
            <TrustItem icon=Ic::Lock text="Files never leave your device"/>
            <TrustItem icon=Ic::WifiOff text="No network needed to convert"/>
            <TrustItem icon=Ic::Code text="Free and open source"/>
        </ul>
    }
}

#[component]
fn TrustItem(icon: Ic, text: &'static str) -> impl IntoView {
    view! {
        <li class="flex items-center justify-center gap-2">
            <Icon icon=icon class="size-4 text-subtle"/>
            {text}
        </li>
    }
}

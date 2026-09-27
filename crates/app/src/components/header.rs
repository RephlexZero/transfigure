use leptos::prelude::*;

use super::icons::{GithubMark, Ic, Icon, Logo};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-40 border-b border-line/70 bg-page/80 backdrop-blur-md supports-[backdrop-filter]:bg-page/70">
            <div class="mx-auto max-w-6xl px-4 sm:px-6 h-14 flex items-center justify-between gap-4">
                <a href="/" class="flex items-center gap-2.5 rounded-md -mx-1 px-1" aria-label="Transfigure home">
                    <Logo class="size-[22px] text-accent"/>
                    <span class="text-[15px] font-semibold tracking-tight">"Transfigure"</span>
                </a>

                <nav class="flex items-center gap-1 text-sm" aria-label="Main">
                    <a href="#how" class="btn btn-sm btn-ghost hidden md:inline-flex">"How it works"</a>
                    <a href="#formats" class="btn btn-sm btn-ghost hidden md:inline-flex">"Formats"</a>
                    <a href="#faq" class="btn btn-sm btn-ghost hidden md:inline-flex">"FAQ"</a>
                    <span class="hidden md:block w-px h-5 bg-line mx-2" aria-hidden="true"></span>
                    <a
                        href="https://github.com/RephlexZero/transfigure"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="icon-btn size-9"
                        aria-label="Source code on GitHub"
                        title="Source code on GitHub"
                    >
                        <GithubMark/>
                    </a>
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="btn btn-sm btn-secondary ml-1"
                    >
                        <Icon icon=Ic::Heart class="size-3.5 text-accent"/>
                        "Support"
                    </a>
                </nav>
            </div>
        </header>
    }
}

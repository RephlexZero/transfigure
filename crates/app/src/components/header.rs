use leptos::prelude::*;

use super::icons::{GithubMark, Logo};

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="flex items-center justify-between gap-6 h-16 border-b-2 border-ink">
            <a href="/" class="flex items-center gap-2.5 rounded" aria-label="Transfigure home">
                <Logo class="size-[30px]"/>
                <span class="text-[23px] font-bold tracking-[-0.025em]">"Transfigure"</span>
            </a>

            <p class="hidden xl:block label">"private file conversion, in your browser"</p>

            <nav class="flex items-center gap-1 sm:gap-5" aria-label="Main">
                <a href="#how" class="nav-link hidden md:inline-flex">"how it works"</a>
                <a href="#formats" class="nav-link hidden md:inline-flex">"formats"</a>
                <a href="#faq" class="nav-link hidden md:inline-flex">"questions"</a>
                <a
                    href="https://github.com/RephlexZero/transfigure"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="icon-btn"
                    aria-label="Source code on GitHub"
                    title="Source code on GitHub"
                >
                    <GithubMark/>
                </a>
                <a
                    href="https://ko-fi.com/rephlexzero"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="nav-link text-ink font-semibold border-b-[3px] border-res h-7 ml-1"
                >
                    "support"
                </a>
            </nav>
        </header>
    }
}

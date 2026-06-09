use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 backdrop-blur bg-base-300/85 border-b-2 hairline">
            <div class="container mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between h-14">
                <a href="/" class="flex items-center gap-3">
                    // Alembic mark: triangle-in-circle, drawn inline so it ships with the binary.
                    <svg class="w-6 h-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <circle cx="12" cy="12" r="10"/>
                        <path d="M12 5.5 18 16.5 H6 Z" stroke-linejoin="round"/>
                        <line x1="9" y1="13" x2="15" y2="13"/>
                    </svg>
                    <span class="text-sm font-bold uppercase tracking-[0.35em]">"Transfigure"</span>
                    <span class="hidden md:inline text-[9px] uppercase tracking-[0.25em] text-base-content/40 border hairline px-2 py-1">
                        "Local-first"
                    </span>
                </a>
                <nav class="flex items-center gap-5 text-[11px] uppercase tracking-[0.18em]">
                    <a href="#how" class="text-base-content/55 hover:text-primary transition-colors hidden sm:inline">"Method"</a>
                    <a href="#formats" class="text-base-content/55 hover:text-primary transition-colors hidden sm:inline">"Formats"</a>
                    <a
                        href="https://ko-fi.com/rephlexzero"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-base-content/55 hover:text-primary transition-colors"
                    >"Support"</a>
                    <a
                        href="https://github.com/RephlexZero/transfigure"
                        target="_blank"
                        rel="noopener noreferrer"
                        aria-label="Source on GitHub"
                        class="text-base-content/55 hover:text-primary transition-colors"
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
                        </svg>
                    </a>
                </nav>
            </div>
        </header>
    }
}

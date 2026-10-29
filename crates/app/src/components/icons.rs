use leptos::prelude::*;

/// Stroke icons on a 24×24 grid (paths adapted from Lucide, ISC licence).
/// Kept as markup strings so each icon is one line and they all share the
/// same stroke settings from `Icon`.
#[derive(Clone, Copy)]
pub enum Ic {
    Download,
    X,
    ArrowRight,
    Plus,
    Alert,
    RotateCcw,
}

impl Ic {
    fn paths(self) -> &'static str {
        match self {
            Ic::Download => {
                r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/><path d="M12 15V3"/>"#
            }
            Ic::X => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
            Ic::ArrowRight => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
            Ic::Plus => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
            Ic::Alert => {
                r#"<circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/>"#
            }
            Ic::RotateCcw => {
                r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>"#
            }
        }
    }
}

#[component]
pub fn Icon(icon: Ic, #[prop(default = "size-4")] class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            inner_html=icon.paths()
        ></svg>
    }
}

/// The Transfigure mark: an alchemical triangle inside a circle, the circle
/// in source ink and the triangle in result ink, overprinted slightly off
/// register.
#[component]
pub fn Logo(#[prop(default = "size-7")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=format!("{class} isolate shrink-0") viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <circle
                cx="11.4"
                cy="12.6"
                r="10"
                stroke-width="1.8"
                class="stroke-src mix-blend-multiply dark:mix-blend-screen"
            />
            <path
                d="M12 5.5 18 16.5H6Z"
                stroke-width="1.4"
                stroke-linejoin="round"
                class="fill-res stroke-res mix-blend-multiply dark:mix-blend-screen"
            />
            <path d="M9 13h6" stroke-width="1.6" class="stroke-ink"/>
        </svg>
    }
}

#[component]
pub fn GithubMark(#[prop(default = "size-[18px]")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
        </svg>
    }
}

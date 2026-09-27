use leptos::prelude::*;

use crate::utils::Category;

/// Stroke icons on a 24×24 grid (paths adapted from Lucide, ISC licence).
/// Kept as markup strings so each icon is one line and they all share the
/// same stroke settings from `Icon`.
#[derive(Clone, Copy)]
pub enum Ic {
    Upload,
    Download,
    X,
    ArrowRight,
    ArrowDown,
    Plus,
    Alert,
    Lock,
    WifiOff,
    Code,
    Heart,
    Cpu,
    Server,
    Monitor,
    RotateCcw,
    Image,
    Music,
    FileText,
    Sheet,
    Braces,
    Binary,
    File,
}

impl Ic {
    fn paths(self) -> &'static str {
        match self {
            Ic::Upload => {
                r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m17 8-5-5-5 5"/><path d="M12 3v12"/>"#
            }
            Ic::Download => {
                r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/><path d="M12 15V3"/>"#
            }
            Ic::X => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
            Ic::ArrowRight => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
            Ic::ArrowDown => r#"<path d="M12 5v14"/><path d="m19 12-7 7-7-7"/>"#,
            Ic::Plus => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
            Ic::Alert => {
                r#"<circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/>"#
            }
            Ic::Lock => {
                r#"<rect width="18" height="11" x="3" y="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#
            }
            Ic::WifiOff => {
                r#"<path d="M12 20h.01"/><path d="M8.5 16.43a5 5 0 0 1 7 0"/><path d="M5 12.86a10 10 0 0 1 5.17-2.69"/><path d="M19 12.86a10 10 0 0 0-2-1.52"/><path d="M2 8.82a15 15 0 0 1 4.18-2.64"/><path d="M22 8.82a15 15 0 0 0-11.29-3.76"/><path d="m2 2 20 20"/>"#
            }
            Ic::Code => r#"<path d="m16 18 6-6-6-6"/><path d="m8 6-6 6 6 6"/>"#,
            Ic::Heart => {
                r#"<path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/>"#
            }
            Ic::Cpu => {
                r#"<rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2M15 20v2M2 15h2M2 9h2M20 15h2M20 9h2M9 2v2M9 20v2"/>"#
            }
            Ic::Server => {
                r#"<rect width="20" height="8" x="2" y="2" rx="2"/><rect width="20" height="8" x="2" y="14" rx="2"/><path d="M6 6h.01M6 18h.01"/>"#
            }
            Ic::Monitor => {
                r#"<rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8M12 17v4"/>"#
            }
            Ic::RotateCcw => {
                r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>"#
            }
            Ic::Image => {
                r#"<rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.09-3.09a2 2 0 0 0-2.82 0L6 21"/>"#
            }
            Ic::Music => {
                r#"<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>"#
            }
            Ic::FileText => {
                r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M16 13H8M16 17H8M10 9H8"/>"#
            }
            Ic::Sheet => {
                r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18M3 15h18M9 9v12"/>"#
            }
            Ic::Braces => {
                r#"<path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5a2 2 0 0 0 2 2h1"/><path d="M16 21h1a2 2 0 0 0 2-2v-5a2 2 0 0 1 2-2 2 2 0 0 1-2-2V5a2 2 0 0 0-2-2h-1"/>"#
            }
            Ic::Binary => {
                r#"<rect width="4" height="6" x="14" y="14" rx="2"/><rect width="4" height="6" x="6" y="4" rx="2"/><path d="M6 20h4M14 10h4M6 14h2v6M14 4h2v6"/>"#
            }
            Ic::File => {
                r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>"#
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

/// The Transfigure mark: an alchemical triangle inside a circle.
#[component]
pub fn Logo(#[prop(default = "size-6")] class: &'static str) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <path d="M12 5.5 18 16.5H6Z" stroke-linejoin="round"/>
            <path d="M9 13h6"/>
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

/// Icon and tint for a file family, shared by the file list and format grid.
pub fn category_style(cat: Category) -> (Ic, &'static str) {
    match cat {
        Category::Image => (Ic::Image, "bg-sky-500/10 text-sky-700 dark:text-sky-300"),
        Category::Audio => (
            Ic::Music,
            "bg-fuchsia-500/10 text-fuchsia-700 dark:text-fuchsia-300",
        ),
        Category::Document => (
            Ic::FileText,
            "bg-orange-500/10 text-orange-700 dark:text-orange-300",
        ),
        Category::Data => (
            Ic::Sheet,
            "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
        ),
        Category::Config => (
            Ic::Braces,
            "bg-violet-500/10 text-violet-700 dark:text-violet-300",
        ),
        Category::Encoding => (
            Ic::Binary,
            "bg-stone-500/10 text-stone-600 dark:text-stone-300",
        ),
        Category::Other => (
            Ic::File,
            "bg-stone-500/10 text-stone-600 dark:text-stone-300",
        ),
    }
}

/// Rounded tile holding a category icon.
#[component]
pub fn CategoryTile(
    cat: Category,
    #[prop(default = "size-9")] size: &'static str,
) -> impl IntoView {
    let (icon, tint) = category_style(cat);
    view! {
        <span class=format!("{size} shrink-0 inline-flex items-center justify-center rounded-lg {tint}")>
            <Icon icon=icon class="size-[18px]"/>
        </span>
    }
}

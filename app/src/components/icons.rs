//! Inline SVG icons and the Addict Order lockbox mark.

use leptos::prelude::*;

/// Compact lockbox mark used in the top bar and login header.
#[component]
pub fn BrandMark() -> impl IntoView {
    view! {
        <svg class="brand-mark" viewBox="0 0 48 48" width="28" height="28" aria-hidden="true">
            <rect width="48" height="48" rx="6" fill="#1D4ED8" />
            <rect x="6" y="17" width="36" height="25" fill="#3B82F6" stroke="#111111" stroke-width="2" />
            <rect x="6" y="17" width="36" height="8" fill="#FACC15" stroke="#111111" stroke-width="2" />
            <path d="M16 17V12C16 4 32 4 32 12V17" fill="none" stroke="#FACC15" stroke-width="5" />
            <path d="M16 17V12C16 4 32 4 32 12V17" fill="none" stroke="#111111" stroke-width="1.6" />
            <circle cx="24" cy="30" r="3" fill="#111111" />
            <rect x="23" y="31" width="2" height="5" fill="#111111" />
        </svg>
    }
}

/// Full master artwork used on the login brand panel.
#[component]
pub fn BrandArt() -> impl IntoView {
    view! {
        <div class="login__art" inner_html=include_str!("../../assets/icon-master.svg")></div>
    }
}

#[component]
pub fn IconSearch() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-4-4" />
        </svg>
    }
}

#[component]
pub fn IconSettings() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.12 2.12-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1 1.56V21h-3v-.09a1.7 1.7 0 0 0-1-1.56 1.7 1.7 0 0 0-1.88.34l-.06.06-2.12-2.12.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.56-1H3v-3h.09a1.7 1.7 0 0 0 1.56-1 1.7 1.7 0 0 0-.34-1.88l-.06-.06 2.12-2.12.06.06A1.7 1.7 0 0 0 8.4 5.6a1.7 1.7 0 0 0 1-1.56V4h3v.09a1.7 1.7 0 0 0 1 1.56 1.7 1.7 0 0 0 1.88-.34l.06-.06 2.12 2.12-.06.06A1.7 1.7 0 0 0 19.4 9a1.7 1.7 0 0 0 1.56 1H21v3h-.09a1.7 1.7 0 0 0-1.51 1Z" />
        </svg>
    }
}

#[component]
pub fn IconLogOut() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
            <path d="m16 17 5-5-5-5" />
            <path d="M21 12H9" />
        </svg>
    }
}

#[component]
pub fn IconLock() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="4" y="10" width="16" height="11" rx="1" />
            <path d="M8 10V7a4 4 0 0 1 8 0v3" />
        </svg>
    }
}

#[component]
pub fn IconPrint() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M6 9V3h12v6" />
            <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
            <rect x="6" y="14" width="12" height="8" />
        </svg>
    }
}

#[component]
pub fn IconCheck() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m5 12 4 4L19 6" />
        </svg>
    }
}

#[component]
pub fn IconWarning() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 3 2 21h20Z" />
            <path d="M12 9v5" />
            <path d="M12 18h.01" />
        </svg>
    }
}

#[component]
pub fn IconX() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="m6 6 12 12M18 6 6 18" />
        </svg>
    }
}

#[component]
pub fn IconArrowRight() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M5 12h14" />
            <path d="m13 6 6 6-6 6" />
        </svg>
    }
}

#[component]
pub fn IconUser() -> impl IntoView {
    view! {
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="8" r="4" />
            <path d="M4 21a8 8 0 0 1 16 0" />
        </svg>
    }
}

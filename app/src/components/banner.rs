//! Dismissible message banner.

use std::time::Duration;

use leptos::prelude::*;

use crate::components::icons::IconX;
use crate::state::{AppState, BannerKind};

#[component]
pub fn BannerHost(state: AppState) -> impl IntoView {
    let auto_state = state.clone();
    Effect::new(move |_| {
        let Some(banner) = auto_state.banner.get() else {
            return;
        };
        if banner.kind != BannerKind::Info {
            return;
        }
        let message = banner.message.clone();
        let state = auto_state.clone();
        let _ = set_timeout_with_handle(
            move || {
                let still_same = state.banner.get_untracked().is_some_and(|current| {
                    current.kind == BannerKind::Info && current.message == message
                });
                if still_same {
                    state.banner.set(None);
                }
            },
            Duration::from_secs(5),
        );
    });

    view! {
        {move || {
            state.banner.get().map(|banner| {
                let class = match banner.kind {
                    BannerKind::Error => "banner banner--error",
                    BannerKind::Warn => "banner banner--warn",
                    BannerKind::Info => "banner banner--info",
                };
                let state = state.clone();
                view! {
                    <div class=class role="status">
                        <span class="banner__text">{banner.message}</span>
                        <button
                            class="banner__close"
                            aria-label="ปิด"
                            on:click=move |_| state.banner.set(None)
                        >
                            <IconX />
                        </button>
                    </div>
                }
            })
        }}
    }
}

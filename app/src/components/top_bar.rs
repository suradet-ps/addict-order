//! Top bar: brand, connection state, signed-in officer, logout.

use leptos::prelude::*;

use crate::api;
use crate::components::icons::{BrandMark, IconLogOut};
use crate::state::{AppState, SettingsTab};

#[component]
pub fn TopBar(state: AppState) -> impl IntoView {
    let health = state.health;
    let open_settings = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Connection);
            state.settings_open.set(true);
        }
    };
    let logout = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                let _ = api::logout().await;
                state.clear_session();
            });
        }
    };

    view! {
        <header class="top-bar">
            <div class="top-bar__left">
                <button
                    class="brand-button"
                    title="ตั้งค่า"
                    aria-label="เปิดการตั้งค่า"
                    aria-haspopup="dialog"
                    on:click=open_settings
                >
                    <BrandMark />
                </button>
                <div class="top-bar__titles">
                    <span class="top-bar__title">"Addict Order"</span>
                    <span class="top-bar__subtitle">"ระบบใบสั่งจ่าย ย.ส. ๒ / ว.จ. ๒"</span>
                    <span
                        class=move || {
                            format!(
                                "status-dot status-dot--{}",
                                health.get().dot_class(),
                            )
                        }
                        role="status"
                        tabindex="0"
                        aria-label=move || health.get().label_th()
                        data-tooltip=move || health.get().label_th()
                    >
                        <span class="status-dot__dot" aria-hidden="true"></span>
                    </span>
                </div>
            </div>
            <div class="top-bar__right">
                {move || {
                    state.session.get().map(|user| {
                        let initial = user
                            .display_name
                            .chars()
                            .next()
                            .or_else(|| user.login_name.chars().next())
                            .map(|c| c.to_string())
                            .unwrap_or_default();
                        view! {
                            <span class="user-chip" title=format!("{} · {}", user.display_name.clone(), user.login_name)>
                                <span class="user-chip__avatar">{initial}</span>
                                <span class="user-chip__name">{user.display_name.clone()}</span>
                            </span>
                        }
                    })
                }}
                <button class="button-secondary top-bar__logout" title="ออกจากระบบ" on:click=logout>
                    <IconLogOut />
                    "ออกจากระบบ"
                </button>
            </div>
        </header>
    }
}

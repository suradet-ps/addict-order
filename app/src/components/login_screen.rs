//! Login gate and startup splash.

use leptos::prelude::*;

use crate::api::{self, ApiErrorKind};
use crate::components::icons::{BrandArt, BrandMark, IconArrowRight, IconLock, IconSettings};
use crate::state::{AppState, Banner, ConnectionHealth, SettingsTab};

#[component]
pub fn LoginScreen(state: AppState) -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let login_busy = RwSignal::new(false);
    let login_error = RwSignal::new(None::<String>);

    let open_settings = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Connection);
            state.settings_open.set(true);
        }
    };

    let on_login = {
        let state = state.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let login_name = username.get();
            if login_name.trim().is_empty() {
                login_error.set(Some("กรุณากรอกชื่อผู้ใช้".to_string()));
                return;
            }
            let login_password = password.get();
            if login_password.is_empty() {
                login_error.set(Some("กรุณากรอกรหัสผ่าน".to_string()));
                return;
            }
            login_busy.set(true);
            login_error.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::login(login_name.trim(), &login_password).await {
                    Ok(user) => {
                        password.set(String::new());
                        state.session.set(Some(user));
                        if let Ok(settings) = api::load_settings().await {
                            state.apply_settings(settings);
                        }
                        if state.hospital.get().name.trim().is_empty()
                            && let Ok(Some(source)) = api::fetch_hospital_source().await
                        {
                            let mut hospital = state.hospital.get();
                            hospital.name = source.name;
                            hospital.phone = source.phone;
                            hospital.fax = source.fax;
                            state.set_hospital(hospital);
                            if !source.address_text.trim().is_empty() {
                                state.banner.set(Some(Banner::warn(format!(
                                    "ตรวจสอบที่อยู่สถานพยาบาลก่อนพิมพ์: {}",
                                    source.address_text
                                ))));
                            }
                        }
                    }
                    Err(err) => {
                        if matches!(
                            err.kind,
                            ApiErrorKind::Connection | ApiErrorKind::NotConfigured
                        ) {
                            state.health.set(ConnectionHealth::Disconnected);
                        }
                        login_error.set(Some(err.message));
                    }
                }
                login_busy.set(false);
            });
        }
    };

    view! {
        <div class="login">
            <aside class="login__brand">
                <div class="login__brand-top">
                    <BrandMark />
                    <div class="login__brand-titles">
                        <span class="login__brand-name">"Addict Order"</span>
                        <span class="login__brand-sub">"ระบบใบสั่งจ่าย ย.ส. ๒ / ว.จ. ๒"</span>
                    </div>
                </div>
                <div class="login__stage">
                    <h1 class="login__headline">"ใบสั่งจ่ายที่ตรวจสอบได้"</h1>
                    <p class="login__lede">
                        "ค้นผู้ป่วยจาก HOSxP ตรวจรายการยา และพิมพ์ใบสั่งจ่ายตามแบบราชการ"
                    </p>
                    <BrandArt />
                </div>
                <p class="login__footnote">"HOSxP read-only · เอกสารมาตรฐาน กองควบคุมวัตถุเสพติด"</p>
            </aside>

            <main class="login__main">
                <div class="login__toolbar">
                    <button class="button-ghost" on:click=open_settings>
                        <IconSettings />
                        "ตั้งค่า"
                    </button>
                </div>

                <div class="login__card">
                    <div class="login__seal"></div>
                    <header class="login__card-head">
                        <h2 class="login__card-title">"เข้าสู่ระบบ"</h2>
                        <p class="login__card-sub">"ใช้บัญชี officer ของ HOSxP"</p>
                    </header>

                    {move || {
                        (!state.configured.get()).then(|| {
                            view! {
                                <div class="login__notice" role="status">
                                    <div>
                                        <b>"ยังไม่ได้ตั้งค่าการเชื่อมต่อ HOSxP"</b>
                                        <small>"เปิดการตั้งค่าเพื่อเชื่อมต่อก่อนเข้าสู่ระบบ"</small>
                                    </div>
                                    <button class="button-secondary" on:click=open_settings>
                                        "ตั้งค่าการเชื่อมต่อ"
                                    </button>
                                </div>
                            }
                        })
                    }}

                    <form class="login__form" on:submit=on_login>
                        <label class="field">
                            <span class="field__label">"ชื่อผู้ใช้"</span>
                            <input
                                class="text-input"
                                type="text"
                                autocomplete="username"
                                prop:value=move || username.get()
                                on:input=move |ev| username.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="field">
                            <span class="field__label">"รหัสผ่าน"</span>
                            <input
                                class="text-input"
                                type="password"
                                autocomplete="current-password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </label>

                        {move || {
                            login_error.get().map(|message| {
                                view! {
                                    <p class="login__error" role="alert">
                                        <IconLock />
                                        {message}
                                    </p>
                                }
                            })
                        }}

                        <button
                            type="submit"
                            class="button-primary login__submit"
                            disabled=move || !state.configured.get() || login_busy.get()
                        >
                            <IconArrowRight />
                            {move || {
                                if login_busy.get() {
                                    "กำลังเข้าสู่ระบบ..."
                                } else {
                                    "เข้าสู่ระบบ"
                                }
                            }}
                        </button>
                    </form>
                </div>
            </main>
        </div>
    }
}

#[component]
pub fn StartupSplash() -> impl IntoView {
    view! {
        <div class="startup">
            <BrandMark />
            <p>"กำลังเตรียมระบบ..."</p>
        </div>
    }
}

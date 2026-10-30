//! Addict Order frontend (Leptos 0.8, CSR).

// AppState is a Copy handle: every field is a reactive signal handle. The
// explicit `.clone()` calls are kept because they document ownership at each
// UI boundary and match the sibling er-refill code style.
#![allow(clippy::clone_on_copy)]

pub mod actions;
pub mod api;
pub mod components;
pub mod state;

use std::time::Duration;

use leptos::prelude::*;

use components::banner::BannerHost;
use components::command_bar::CommandBar;
use components::dialogs::Dialogs;
use components::document_sheet::DocumentSheet;
use components::empty_state::{CatalogEmpty, EmptyDesk};
use components::inspector::Inspector;
use components::login_screen::{LoginScreen, StartupSplash};
use components::settings_modal::SettingsModal;
use components::top_bar::TopBar;
use state::{AppState, Banner, ConnectionHealth};

pub fn run() {
    leptos::mount::mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();

    let load_state = state.clone();
    leptos::task::spawn_local(async move {
        load_initial_state(load_state).await;
    });
    poll_health_loop(state.clone());

    let shell_state = state.clone();
    let stage_state = state.clone();
    view! {
        <div class="app">
            {move || {
                let state = shell_state.clone();
                if !state.auth_checked.get() {
                    view! { <StartupSplash /> }.into_any()
                } else if state.session.get().is_none() {
                    view! {
                        <LoginScreen state=state.clone() />
                    }.into_any()
                } else {
                    let class_stage = stage_state.clone();
                    let content_stage = stage_state.clone();
                    view! {
                        <TopBar state=state.clone() />
                        <CommandBar state=state.clone() />
                        <main class=move || {
                            if class_stage.selected_patient.get().is_some() { "stage" } else { "stage stage--empty" }
                        }>
                            {move || {
                                let stage = content_stage.clone();
                                if stage.catalog_is_empty() {
                                    view! { <CatalogEmpty state=stage.clone() /> }.into_any()
                                } else if stage.selected_patient.get().is_none() {
                                    view! { <EmptyDesk state=stage.clone() /> }.into_any()
                                } else {
                                    view! {
                                        <DocumentSheet state=stage.clone() />
                                        <Inspector state=stage.clone() />
                                    }.into_any()
                                }
                            }}
                        </main>
                    }.into_any()
                }
            }}
            <Dialogs state=state.clone() />
            <SettingsModal state=state.clone() />
            <BannerHost state=state.clone() />
        </div>
    }
}

async fn load_initial_state(state: AppState) {
    let configured = api::connection_status().await;
    state.configured.set(configured);

    match api::load_settings().await {
        Ok(settings) => state.apply_settings(settings),
        Err(error) => state.banner.set(Some(Banner::error(error.message))),
    }

    if configured {
        if let Ok(summary) = api::connection_summary().await {
            state.connection_summary.set(summary);
        }
        match api::connection_health().await {
            Ok(health) => state.health.set(health),
            Err(_) => state.health.set(ConnectionHealth::Disconnected),
        }
    } else {
        state.health.set(ConnectionHealth::Unconfigured);
    }

    if let Ok(Some(user)) = api::current_session().await {
        state.session.set(Some(user));
        if state.hospital.get().name.trim().is_empty()
            && let Ok(Some(source)) = api::fetch_hospital_source().await
        {
            let mut hospital = state.hospital.get();
            hospital.name = source.name;
            hospital.phone = source.phone;
            hospital.fax = source.fax;
            state.set_hospital(hospital);
        }
    }
    state.auth_checked.set(true);
}

fn poll_health_loop(state: AppState) {
    let next_state = state.clone();
    let _ = set_timeout_with_handle(
        move || {
            let poll_state = next_state.clone();
            leptos::task::spawn_local(async move {
                if let Ok(health) = api::connection_health().await {
                    poll_state.health.set(health);
                }
            });
            poll_health_loop(next_state);
        },
        Duration::from_secs(30),
    );
}

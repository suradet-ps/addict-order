//! Tauri 2 shell for Addict Order. Thin command adapters only.

mod commands;
mod error;
mod settings;
mod state;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let settings_path = config_dir.join(settings::SETTINGS_FILE_NAME);
            let loaded = tauri::async_runtime::block_on(settings::load(&settings_path));
            let app_settings = match loaded {
                Ok(app_settings) => app_settings,
                Err(error) => {
                    error::dev_log(&format!("settings load failed, using defaults: {error}"));
                    settings::AppSettings::default()
                }
            };
            let state = AppState::new(config_dir, app_settings);
            let monitor_state = state.clone();
            tauri::async_runtime::spawn(async move {
                commands::run_health_monitor(monitor_state).await;
            });
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection_status,
            commands::connection_health,
            commands::connection_summary,
            commands::test_connection,
            commands::connect,
            commands::login,
            commands::logout,
            commands::current_session,
            commands::load_settings,
            commands::save_hospital,
            commands::save_catalog,
            commands::search_patients,
            commands::fetch_patient,
            commands::fetch_visits,
            commands::fetch_narcotic_orders,
            commands::search_drugs,
            commands::search_doctors,
            commands::fetch_hospital_source,
        ])
        .run(tauri::generate_context!())
        .expect("invariant: Tauri cannot launch a GUI window");
}

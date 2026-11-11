//! Tauri 2 shell for Addict Order. Thin command adapters only.

mod commands;
mod error;
mod settings;
mod state;

use state::AppState;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

/// Fit the main window inside the monitor work area, then center it there.
/// A fixed 1280x900 window does not fit on 1366x768 or 1600x900 displays once
/// the taskbar and title bar are taken into account.
fn fit_and_center_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        error::dev_log("main window not found for centering");
        return;
    };

    let monitor = match window.current_monitor() {
        Ok(Some(monitor)) => monitor,
        Ok(None) => match window.primary_monitor() {
            Ok(Some(monitor)) => monitor,
            Ok(None) => {
                error::dev_log("no monitor found for centering");
                return;
            }
            Err(error) => {
                error::dev_log(&format!("primary monitor query failed: {error}"));
                return;
            }
        },
        Err(error) => {
            error::dev_log(&format!("current monitor query failed: {error}"));
            return;
        }
    };

    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();

    // `set_size` controls the client area on Windows, while `set_position`
    // places the outer window. Account for the title bar and borders so the
    // visible window is centered and does not spill past the taskbar.
    let (outer_size, inner_size) = match (window.outer_size(), window.inner_size()) {
        (Ok(outer), Ok(inner)) => (outer, inner),
        (Err(error), _) | (_, Err(error)) => {
            error::dev_log(&format!("window size query failed: {error}"));
            return;
        }
    };
    let decoration_width = outer_size.width.saturating_sub(inner_size.width);
    let decoration_height = outer_size.height.saturating_sub(inner_size.height);
    let margin = (24.0 * scale).round().max(0.0) as u32;
    let max_inner_width = work_area
        .size
        .width
        .saturating_sub(margin.saturating_mul(2).saturating_add(decoration_width))
        .max(1);
    let max_inner_height = work_area
        .size
        .height
        .saturating_sub(margin.saturating_mul(2).saturating_add(decoration_height))
        .max(1);
    let inner_width = ((1280.0 * scale).round() as u32)
        .min(max_inner_width)
        .max(1);
    let inner_height = ((900.0 * scale).round() as u32)
        .min(max_inner_height)
        .max(1);

    if let Err(error) = window.set_size(PhysicalSize::new(inner_width, inner_height)) {
        error::dev_log(&format!("window resize failed: {error}"));
        return;
    }

    let outer_size = match window.outer_size() {
        Ok(size) => size,
        Err(error) => {
            error::dev_log(&format!("window outer size query failed: {error}"));
            return;
        }
    };
    let x =
        work_area.position.x + ((work_area.size.width.saturating_sub(outer_size.width)) / 2) as i32;
    let y = work_area.position.y
        + ((work_area.size.height.saturating_sub(outer_size.height)) / 2) as i32;
    if let Err(error) = window.set_position(PhysicalPosition::new(x, y)) {
        error::dev_log(&format!("window center failed: {error}"));
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Fit and center while the window is still hidden, then show it at
            // its final size so it never visibly jumps after launch.
            fit_and_center_main_window(app.handle());
            if let Some(window) = app.get_webview_window("main")
                && let Err(error) = window.show()
            {
                error::dev_log(&format!("window show failed: {error}"));
            }

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

pub mod commands;
pub mod config;
pub mod connection;
pub mod error;
pub mod i18n;
pub mod network;
pub mod server;
pub mod telemetry;
pub mod text;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{
    menu::{Menu, MenuItem},
    Manager, RunEvent, WindowEvent,
};
use tokio::sync::Mutex;
use tracing::info;

use crate::commands::AppState;
use crate::config::DropVoiceConfig;
use crate::telemetry::{cleanup_old_logs, init_logging, LogGuard};

/// Entry point invoked by `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialise structured logging first so the rest of startup is observable.
    let log_guard = init_logging();
    info!("DropVoice desktop backend starting up");

    let config = match DropVoiceConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to load config, using defaults: {e}");
            DropVoiceConfig::default()
        }
    };

    let retention_days = config.telemetry.log_retention_days;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // Spawn a background cleanup of old logs (best-effort).
            tauri::async_runtime::spawn(async move {
                cleanup_old_logs(retention_days);
            });

            // Resolve the directory that holds mobile.html and other static
            // resources. We use Tauri's resource_dir in production and a
            // workspace-relative path in development.
            let resource_dir = resolve_resource_dir(app.handle());

            // Build the shared application state.
            let app_state = AppState::new(config.clone(), resource_dir);
            app.manage(app_state);

            // Stash the LogGuard so it lives for the duration of the app.
            app.manage(LogGuardHolder {
                guard: Arc::new(Mutex::new(Some(log_guard))),
            });

            // Build the system-tray menu (Show / Hide / Quit).
            // Use i18n translations based on config language (spec 15 §5 constraint 5).
            let tray = i18n::load_tray_translations(&config.app.language);
            let show_item = MenuItem::with_id(app, "show", tray.show, true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", tray.hide, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", tray.quit, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::with_id("main-tray")
                .tooltip("DropVoice")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        info!("quit requested from tray");
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            tray_builder.build(app)?;
            info!("tray icon installed");

            Ok(())
        })
        .on_window_event(|window, event| {
            // Close button hides to tray instead of exiting (per spec).
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                info!("window close intercepted, hidden to tray");
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::get_connection_info,
            commands::server::inject_text,
            commands::server::save_token,
            commands::server::issue_connection_token,
            commands::server::validate_credential,
            commands::server::refresh_pairing_token,
            commands::server::get_pairing_token,
            commands::server::register_client,
            commands::server::unregister_client,
            commands::settings::get_settings,
            commands::settings::set_language,
            commands::settings::set_theme,
            commands::settings::set_input_delay,
            commands::window::minimize_to_tray,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let RunEvent::ExitRequested { code, .. } = event {
                info!(?code, "exit requested");
            }
        });
}

/// Holds the `LogGuard` so it isn't dropped until the app exits.
struct LogGuardHolder {
    #[allow(dead_code)]
    guard: Arc<Mutex<Option<LogGuard>>>,
}

/// 解析资源目录（保留兼容；WebRTC 主路径下 PWA 经 CDN 托管，不再需要本地静态文件）。
fn resolve_resource_dir(_app: &tauri::AppHandle) -> PathBuf {
    // 返回 manifest 目录（不再查找 mobile.html；旧静态文件服务已删除）。
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

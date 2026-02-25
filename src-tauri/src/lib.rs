// Modules
mod text;
mod network;
mod server;
mod commands;
mod i18n;

use commands::ServerState;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

pub(crate) fn build_tray_menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, String> {
    use tauri::menu::{Menu, MenuItem};
    use crate::i18n::get_tray_texts;

    // Get current language
    let lang = app.state::<crate::commands::ServerState>()
        .language
        .lock()
        .unwrap()
        .clone();

    let texts = get_tray_texts(&lang);

    let show = MenuItem::with_id(app, "show", texts.show, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let hide = MenuItem::with_id(app, "hide", texts.hide, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", texts.quit, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    Menu::with_items(app, &[&show, &hide, &quit])
        .map_err(|e| e.to_string())
}

pub(crate) fn rebuild_tray_menu(app: &tauri::AppHandle) -> Result<(), String> {
    let menu = build_tray_menu(app)?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))
            .map_err(|e| format!("Failed to set tray menu: {}", e))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .manage(ServerState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--hidden"])
                .build(),
        )
        .setup(|app| {
            // Only enable auto-start in release builds to avoid registering
            // development paths in the Windows registry
            #[cfg(not(debug_assertions))]
            if let Ok(false) = app.autolaunch().is_enabled() {
                if let Err(e) = app.autolaunch().enable() {
                    tracing::error!("Failed to enable auto-start: {}", e);
                }
            }
            let app_handle = app.handle();
            let initial_lang = crate::commands::load_language_preference(&app_handle);
            *app.state::<ServerState>().language.lock().unwrap() = initial_lang;
            let minimize_to_tray = crate::commands::load_minimize_to_tray_preference(&app_handle);
            app.state::<ServerState>()
                .minimize_to_tray_enabled
                .store(minimize_to_tray, std::sync::atomic::Ordering::SeqCst);
            let initial_delay = crate::commands::load_input_delay_preference(&app_handle);
            *app.state::<ServerState>().input_delay_ms.lock().unwrap() = initial_delay;
            let menu = build_tray_menu(&app_handle)?;

            let mut builder = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.minimize();
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                builder = builder.icon(icon);
            }

            builder.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let minimize_to_tray = window
                    .app_handle()
                    .state::<ServerState>()
                    .minimize_to_tray_enabled
                    .load(std::sync::atomic::Ordering::SeqCst);

                if minimize_to_tray {
                    api.prevent_close();
                    let _ = window.minimize();
                    let _ = window.hide();
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_server,
            commands::stop_server,
            commands::get_connection_info,
            commands::set_language,
            commands::get_language,
            commands::get_system_language,
            commands::set_theme,
            commands::get_theme,
            commands::set_lan_warning_dismissed,
            commands::get_lan_warning_dismissed,
            commands::set_minimize_to_tray,
            commands::get_minimize_to_tray,
            commands::get_minimize_to_tray_visible,
            commands::set_input_delay,
            commands::get_input_delay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    run()
}

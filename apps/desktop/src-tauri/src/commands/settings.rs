use tauri::State;
use tracing::info;

use crate::commands::{AppState, Settings};
use crate::error::{AppError, AppResult};

/// Returns the user-facing settings snapshot.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    let cfg = state.config.read().await;
    Ok(Settings {
        language: cfg.app.language.clone(),
        theme: cfg.app.theme.clone(),
        delay_ms: cfg.injection.delay_ms,
        port: cfg.server.port,
        max_text_length: cfg.injection.max_text_length,
        minimize_to_tray: cfg.window.minimize_to_tray,
    })
}

/// Updates the UI language and persists the change.
#[tauri::command]
pub async fn set_language(state: State<'_, AppState>, language: String) -> AppResult<()> {
    // Spec 15 declares SUPPORTED_LANGUAGES = ['en', 'zh', 'zh-TW', 'ja'].
    let valid = matches!(language.as_str(), "en" | "zh" | "zh-TW" | "ja");
    if !valid {
        return Err(AppError::InvalidLanguage { lang: language });
    }

    {
        let mut cfg = state.config.write().await;
        cfg.app.language = language.clone();
        cfg.save()?;
    }
    info!(language = %language, "language updated");
    Ok(())
}

/// Updates the UI theme and persists the change.
#[tauri::command]
pub async fn set_theme(state: State<'_, AppState>, theme: String) -> AppResult<()> {
    let valid = matches!(theme.as_str(), "light" | "dark" | "system");
    if !valid {
        return Err(AppError::InvalidTheme { theme });
    }

    {
        let mut cfg = state.config.write().await;
        cfg.app.theme = theme.clone();
        cfg.save()?;
    }
    info!(theme = %theme, "theme updated");
    Ok(())
}

/// Updates the inter-keystroke injection delay (milliseconds) and persists it.
#[tauri::command]
pub async fn set_input_delay(state: State<'_, AppState>, delay_ms: u64) -> AppResult<()> {
    // Sanity bound: 0–5000 ms. Anything larger would make typing unusable.
    if delay_ms > 5000 {
        return Err(AppError::Internal(format!(
            "delay_ms {delay_ms} exceeds maximum of 5000"
        )));
    }

    {
        let mut cfg = state.config.write().await;
        cfg.injection.delay_ms = delay_ms;
        cfg.save()?;
    }
    info!(delay_ms, "input delay updated");
    Ok(())
}

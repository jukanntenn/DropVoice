use tauri::Window;
use tracing::info;

use crate::error::{AppError, AppResult};

/// Hides the window so the app continues running in the system tray.
#[tauri::command]
pub fn minimize_to_tray(window: Window) -> AppResult<()> {
    window
        .hide()
        .map_err(|e| AppError::Internal(format!("failed to hide window: {e}")))?;
    info!("window hidden to tray");
    Ok(())
}

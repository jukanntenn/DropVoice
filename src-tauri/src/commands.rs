use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::fs;
use tauri::{State, AppHandle, Manager};
use tracing::{info, error, warn};
use crate::server::http::{self, ResourcePaths};
use crate::server::websocket::ConnectionManager;
use crate::network::discovery;
use crate::i18n::{validate_language, get_system_language as get_system_language_internal};
use crate::rebuild_tray_menu;

/// Server state managed by Tauri
pub struct ServerState {
    is_running: Arc<AtomicBool>,
    port: Arc<Mutex<Option<u16>>>,
    connection_manager: ConnectionManager,
    pub language: Arc<Mutex<String>>,
    pub minimize_to_tray_enabled: Arc<AtomicBool>,
    pub input_delay_ms: Arc<Mutex<u64>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            port: Arc::new(Mutex::new(None)),
            connection_manager: ConnectionManager::new(),
            language: Arc::new(Mutex::new("en".to_string())),
            minimize_to_tray_enabled: Arc::new(AtomicBool::new(true)),
            input_delay_ms: Arc::new(Mutex::new(10)),
        }
    }
}

/// Start the WebSocket/HTTP server
#[tauri::command]
pub async fn start_server(
    state: State<'_, ServerState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let default_port = 38425;

    // Resolve the resource directory path
    let resource_dir = resolve_resource_dir(&app_handle);
    let paths = Arc::new(ResourcePaths::new(resource_dir));

    // Log the resource directory for debugging
    info!("Resource directory: {:?}", paths.base_path);

    // Check if already running
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Server is already running".to_string());
    }

    let connection_manager = state.connection_manager.clone();

    // Initialize connection manager with current input delay
    let initial_delay = *state.input_delay_ms.lock().unwrap();
    connection_manager.set_input_delay(initial_delay);
    let is_running = state.is_running.clone();
    let port_state = state.port.clone();

    let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", default_port)).await {
        Ok(listener) => listener,
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            tokio::net::TcpListener::bind("0.0.0.0:0")
                .await
                .map_err(|err| format!("Failed to bind to port {} or fallback: {}", default_port, err))?
        }
        Err(e) => return Err(format!("Failed to bind to port {}: {}", default_port, e)),
    };

    let bound_port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get bound port: {}", e))?
        .port();

    let url = discovery::get_connection_url(bound_port)
        .map_err(|e| format!("Failed to get connection URL: {}", e))?;

    // Double-check after binding
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Server is already running".to_string());
    }

    // Mark as running only after successful bind
    is_running.store(true, Ordering::SeqCst);
    if let Ok(mut port_lock) = port_state.lock() {
        *port_lock = Some(bound_port);
    }

    info!("Server started on {}", url);

    // Start server in background with the already-bound listener
    let port_state = port_state.clone();
    tokio::spawn(async move {
        if let Err(e) = http::serve_with_listener(listener, connection_manager, paths).await {
            error!("Server error: {}", e);
            is_running.store(false, Ordering::SeqCst);
            if let Ok(mut port_lock) = port_state.lock() {
                *port_lock = None;
            }
        }
    });

    Ok(url)
}

/// Stop the server (note: in this implementation, server runs for the app lifetime)
#[tauri::command]
pub async fn stop_server(state: State<'_, ServerState>) -> Result<(), String> {
    state.is_running.store(false, Ordering::SeqCst);
    if let Ok(mut port_lock) = state.port.lock() {
        *port_lock = None;
    }
    info!("Server stopped");
    Ok(())
}

/// Get connection information
#[tauri::command]
pub async fn get_connection_info(
    state: State<'_, ServerState>,
) -> Result<ConnectionInfo, String> {
    let port = state.port.lock().ok().and_then(|lock| *lock);
    let url = match port {
        Some(port) => discovery::get_connection_url(port)
            .map_err(|e| format!("Failed to get connection URL: {}", e))?,
        None => String::new(),
    };

    Ok(ConnectionInfo {
        is_running: state.is_running.load(Ordering::SeqCst),
        url,
        connection_count: state.connection_manager.get_count(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub is_running: bool,
    pub url: String,
    pub connection_count: usize,
}

/// Resolve the resource directory path
/// Tries multiple fallback locations to find the public folder
fn resolve_resource_dir(app_handle: &AppHandle) -> PathBuf {
    fn matches_candidate(candidate: &PathBuf) -> bool {
        candidate.join("src").join("mobile").join("index.html").exists()
            || candidate.join("mobile").join("index.html").exists()
            || candidate.join("mobile.html").exists()
    }

    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        candidates.push(resource_dir.join("public"));
        candidates.push(resource_dir.join("dist"));
        candidates.push(resource_dir.clone());
        candidates.push(resource_dir.join("_up_").join("public"));
        candidates.push(resource_dir.join("_up_").join("dist"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("public"));
            candidates.push(exe_dir.join("dist"));
            candidates.push(exe_dir.join("resources").join("public"));
            candidates.push(exe_dir.join("resources").join("dist"));
            candidates.push(exe_dir.join("resources"));
            if let Some(parent_dir) = exe_dir.parent() {
                candidates.push(parent_dir.join("public"));
                candidates.push(parent_dir.join("dist"));
                candidates.push(parent_dir.join("resources").join("public"));
                candidates.push(parent_dir.join("resources").join("dist"));
                candidates.push(parent_dir.join("resources"));
                candidates.push(parent_dir.join("_up_").join("public"));
                candidates.push(parent_dir.join("_up_").join("dist"));
            }
        }
    }

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let out_dir = PathBuf::from(out_dir);
        candidates.push(out_dir.join("public"));
        candidates.push(out_dir.join("dist"));
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("public"));
        candidates.push(current_dir.join("dist"));
    }

    for candidate in candidates {
        if matches_candidate(&candidate) {
            info!("Using resource dir: {:?}", candidate);
            return candidate;
        }
    }

    let default_path = PathBuf::from("dist");
    warn!("Could not find mobile entry, using default: {:?}", default_path);
    default_path
}

// ============================================================================
// Language Storage Commands
// ============================================================================

const LANGUAGE_FILE: &str = "language-preference.txt";
const THEME_FILE: &str = "theme-preference.txt";
const LAN_WARNING_FILE: &str = "lan-warning-dismissed.txt";
const MINIMIZE_TO_TRAY_FILE: &str = "minimize-to-tray-enabled.txt";
const INPUT_DELAY_FILE: &str = "input-delay-ms.txt";

fn get_language_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(LANGUAGE_FILE)
}

pub(crate) fn load_language_preference(app: &AppHandle) -> String {
    let path = get_language_path(app);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            match validate_language(trimmed) {
                Ok(_) => trimmed.to_string(),
                Err(_) => {
                    warn!("Invalid language in storage: {}, using system language", trimmed);
                    get_system_language_internal()
                }
            }
        }
        Err(_) => get_system_language_internal(),
    }
}

fn get_theme_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(THEME_FILE)
}

fn get_lan_warning_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(LAN_WARNING_FILE)
}

fn get_minimize_to_tray_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(MINIMIZE_TO_TRAY_FILE)
}

fn get_input_delay_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(INPUT_DELAY_FILE)
}

pub(crate) fn load_minimize_to_tray_preference(app: &AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        let path = get_minimize_to_tray_path(app);
        match fs::read_to_string(&path) {
            Ok(content) => content.trim() == "true",
            Err(_) => true,
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        true
    }
}

#[tauri::command]
pub async fn set_language(app: AppHandle, lang: String, state: State<'_, ServerState>) -> Result<(), String> {
    // Validate language code
    let validated = validate_language(&lang)?;

    // Update state
    *state.language.lock().unwrap() = validated.to_string();

    // Save to persistent storage
    let path = get_language_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            let msg = format!("Failed to create language directory: {}", e);
            error!("{}", msg);
            msg
        })?;
    }
    fs::write(&path, validated).map_err(|e| {
        let msg = format!("Failed to write language preference: {}", e);
        error!("{}", msg);
        msg
    })?;

    if let Err(e) = rebuild_tray_menu(&app) {
        warn!("Failed to rebuild tray menu: {}", e);
    }

    info!("Language preference saved: {}", validated);
    Ok(())
}

#[tauri::command]
pub async fn get_language(app: AppHandle) -> Result<String, String> {
    Ok(load_language_preference(&app))
}

#[tauri::command]
pub async fn get_system_language() -> Result<String, String> {
    Ok(get_system_language_internal())
}

// ============================================================================
// Theme Persistence Commands
// ============================================================================

#[tauri::command]
pub async fn set_theme(app: AppHandle, theme: String) -> Result<(), String> {
    if theme != "light" && theme != "dark" && theme != "system" {
        return Err("Invalid theme value. Must be 'light', 'dark', or 'system'".to_string());
    }

    let path = get_theme_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            let msg = format!("Failed to create theme directory: {}", e);
            error!("{}", msg);
            msg
        })?;
    }

    fs::write(&path, &theme).map_err(|e| {
        let msg = format!("Failed to write theme preference: {}", e);
        error!("{}", msg);
        msg
    })?;

    info!("Theme preference saved: {}", theme);
    Ok(())
}

#[tauri::command]
pub async fn get_theme(app: AppHandle) -> Result<String, String> {
    let path = get_theme_path(&app);

    match fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed == "light" || trimmed == "dark" || trimmed == "system" {
                Ok(trimmed.to_string())
            } else {
                warn!("Invalid theme in storage: {}", trimmed);
                Ok(String::new())
            }
        }
        Err(_) => Ok(String::new()),
    }
}

// ============================================================================
// LAN Warning Dismissal Commands
// ============================================================================

#[tauri::command]
pub async fn set_lan_warning_dismissed(app: AppHandle) -> Result<(), String> {
    let path = get_lan_warning_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            let msg = format!("Failed to create lan warning directory: {}", e);
            error!("{}", msg);
            msg
        })?;
    }

    fs::write(&path, "true").map_err(|e| {
        let msg = format!("Failed to write lan warning dismissal: {}", e);
        error!("{}", msg);
        msg
    })?;

    info!("LAN warning dismissal saved");
    Ok(())
}

#[tauri::command]
pub async fn get_lan_warning_dismissed(app: AppHandle) -> Result<bool, String> {
    let path = get_lan_warning_path(&app);
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content.trim() == "true"),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn set_minimize_to_tray(app: AppHandle, enabled: bool, state: State<'_, ServerState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let path = get_minimize_to_tray_path(&app);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                let msg = format!("Failed to create settings directory: {}", e);
                error!("{}", msg);
                msg
            })?;
        }

        fs::write(&path, if enabled { "true" } else { "false" }).map_err(|e| {
            let msg = format!("Failed to write minimize to tray setting: {}", e);
            error!("{}", msg);
            msg
        })?;

        state.minimize_to_tray_enabled.store(enabled, Ordering::SeqCst);
        info!("Minimize to tray setting saved: {}", enabled);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        let _ = enabled;
        state.minimize_to_tray_enabled.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tauri::command]
pub async fn get_minimize_to_tray(state: State<'_, ServerState>) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(state.minimize_to_tray_enabled.load(Ordering::SeqCst))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = state;
        Ok(true)
    }
}

#[tauri::command]
pub fn get_minimize_to_tray_visible() -> bool {
    #[cfg(target_os = "windows")]
    {
        true
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// ============================================================================
// Input Delay Commands
// ============================================================================

pub(crate) fn load_input_delay_preference(app: &AppHandle) -> u64 {
    let path = get_input_delay_path(app);
    match fs::read_to_string(&path) {
        Ok(content) => {
            match content.trim().parse::<u64>() {
                Ok(delay) if delay >= 1 && delay <= 1000 => delay,
                _ => {
                    warn!("Invalid input delay in storage: {}, using default 10ms", content.trim());
                    10
                }
            }
        }
        Err(_) => 10,
    }
}

#[tauri::command]
pub async fn set_input_delay(app: AppHandle, delay_ms: u64, state: State<'_, ServerState>) -> Result<(), String> {
    if delay_ms < 1 || delay_ms > 1000 {
        return Err("Input delay must be between 1 and 1000 milliseconds".to_string());
    }

    let path = get_input_delay_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            let msg = format!("Failed to create settings directory: {}", e);
            error!("{}", msg);
            msg
        })?;
    }

    fs::write(&path, delay_ms.to_string()).map_err(|e| {
        let msg = format!("Failed to write input delay setting: {}", e);
        error!("{}", msg);
        msg
    })?;

    *state.input_delay_ms.lock().unwrap() = delay_ms;
    state.connection_manager.set_input_delay(delay_ms);
    info!("Input delay setting saved: {}ms", delay_ms);
    Ok(())
}

#[tauri::command]
pub async fn get_input_delay(state: State<'_, ServerState>) -> Result<u64, String> {
    Ok(*state.input_delay_ms.lock().unwrap())
}

#[cfg(test)]
mod language_tests {
    #[test]
    fn test_set_language_valid() {
        // This will be tested via integration tests
        // Unit tests for storage logic would require mocking Tauri's store
    }
}

pub mod migration;

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Root configuration loaded from `dropvoice.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DropVoiceConfig {
    #[serde(default)]
    pub meta: MetaConfig,
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub injection: InjectionConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub updater: UpdaterConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub devices: DevicesConfig,
    /// Per-device identity and pairing state (spec 11 §7.1 / §14).
    #[serde(default)]
    pub device: DeviceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    #[serde(default = "default_today")]
    pub created_at: String,
    #[serde(default = "default_today")]
    pub last_modified: String,
}

fn default_config_version() -> u32 {
    2
}

fn default_today() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            config_version: default_config_version(),
            created_at: default_today(),
            last_modified: default_today(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_version")]
    pub version: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_app_version() -> String {
    "0.2.0".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_app_version(),
            language: default_language(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

fn default_port() -> u16 {
    38425
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_max_connections() -> usize {
    10
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionConfig {
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    #[serde(default = "default_max_text_length")]
    pub max_text_length: usize,
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
}

fn default_delay_ms() -> u64 {
    10
}

fn default_max_text_length() -> usize {
    10000
}

fn default_queue_size() -> usize {
    100
}

impl Default for InjectionConfig {
    fn default() -> Self {
        Self {
            delay_ms: default_delay_ms(),
            max_text_length: default_max_text_length(),
            queue_size: default_queue_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 配对码长度（已固定为 6 位，webrtc-scan-direct-design §3.2）。
    /// 保留此字段仅为 TOML 向后兼容；`generate_pairing_code()` 忽略它。
    #[serde(default = "default_pairing_code_length")]
    #[allow(dead_code)]
    pub pairing_code_length: usize,
    #[serde(default = "default_pairing_expiry")]
    pub pairing_code_expiry_minutes: u64,
    #[serde(default = "default_rate_per_minute")]
    pub rate_limit_per_minute: usize,
    #[serde(default = "default_rate_per_hour")]
    pub rate_limit_per_hour: usize,
}

fn default_pairing_code_length() -> usize {
    6
}

fn default_pairing_expiry() -> u64 {
    5
}

fn default_rate_per_minute() -> usize {
    30
}

fn default_rate_per_hour() -> usize {
    500
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pairing_code_length: default_pairing_code_length(),
            pairing_code_expiry_minutes: default_pairing_expiry(),
            rate_limit_per_minute: default_rate_per_minute(),
            rate_limit_per_hour: default_rate_per_hour(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    #[serde(default = "default_updater_enabled")]
    pub enabled: bool,
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
    #[serde(default = "default_updater_endpoint")]
    pub endpoint: String,
}

fn default_updater_enabled() -> bool {
    true
}

fn default_check_interval() -> u64 {
    24
}

fn default_updater_endpoint() -> String {
    "https://releases.dropvoice.app/update/manifest.json".to_string()
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            enabled: default_updater_enabled(),
            check_interval_hours: default_check_interval(),
            endpoint: default_updater_endpoint(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_telemetry_enabled")]
    pub enabled: bool,
    #[serde(default = "default_telemetry_output")]
    pub output: String,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default = "default_log_retention")]
    pub log_retention_days: u64,
}

fn default_telemetry_enabled() -> bool {
    true
}

fn default_telemetry_output() -> String {
    "file".to_string()
}

fn default_log_dir() -> String {
    "logs".to_string()
}

fn default_log_retention() -> u64 {
    30
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_telemetry_enabled(),
            output: default_telemetry_output(),
            log_dir: default_log_dir(),
            log_retention_days: default_log_retention(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    #[serde(default = "default_window_height")]
    pub height: u32,
    #[serde(default = "default_window_resizable")]
    pub resizable: bool,
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
}

fn default_window_width() -> u32 {
    800
}

fn default_window_height() -> u32 {
    650
}

fn default_window_resizable() -> bool {
    true
}

fn default_minimize_to_tray() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            resizable: default_window_resizable(),
            minimize_to_tray: default_minimize_to_tray(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicesConfig {
    #[serde(default = "default_max_devices")]
    pub max_count: usize,
    #[serde(default = "default_auto_connect")]
    pub auto_connect: bool,
}

fn default_max_devices() -> usize {
    5
}

fn default_auto_connect() -> bool {
    true
}

impl Default for DevicesConfig {
    fn default() -> Self {
        Self {
            max_count: default_max_devices(),
            auto_connect: default_auto_connect(),
        }
    }
}

/// Per-device identity and pairing state (spec 11 §7.1, §6.6, §14).
///
/// `device_id` is a UUID v4 generated once on first launch and never changed.
/// `device_name` is derived from hostname on first launch and then frozen;
/// users may modify it via the settings page.
/// `pairing_token` is the token issued by the pairing server (spec 11 §5.1.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    #[serde(default = "default_device_id")]
    pub device_id: String,
    #[serde(default = "default_device_name")]
    pub device_name: String,
    #[serde(default)]
    pub pairing_token: Option<String>,
    /// Persisted connection tokens issued to mobile clients (B2).
    /// Survives desktop restart so paired mobiles don't need re-pairing.
    #[serde(default)]
    pub connected_tokens: Option<Vec<String>>,
}

fn default_device_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_device_name() -> String {
    hostname_friendly_name()
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_id: default_device_id(),
            device_name: default_device_name(),
            pairing_token: None,
            connected_tokens: None,
        }
    }
}

/// Generate a friendly device name from the hostname (spec 11 §6.6.2).
///
/// - Windows `DESKTOP-XXX` default hostname → brand default "DropVoice Desktop".
/// - Other platforms' hostname (user-customized, usually readable) → use as-is.
/// - No hostname available → brand default.
pub fn hostname_friendly_name() -> String {
    let raw = read_hostname();
    match raw {
        Some(h) if is_unreadable_default(&h) => "DropVoice Desktop".to_string(),
        Some(h) => h,
        None => "DropVoice Desktop".to_string(),
    }
}

fn read_hostname() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("COMPUTERNAME").ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(h) = std::env::var("HOSTNAME") {
            return Some(h);
        }
        std::fs::read_to_string("/etc/hostname")
            .ok()
            .map(|s| s.trim().to_string())
    }
}

fn is_unreadable_default(name: &str) -> bool {
    let upper = name.to_uppercase();
    if let Some(rest) = upper.strip_prefix("DESKTOP-") {
        return rest.len() >= 7 && rest.chars().all(|c| c.is_ascii_alphanumeric());
    }
    false
}

impl DropVoiceConfig {
    /// Returns the platform-specific config directory.
    pub fn config_dir() -> PathBuf {
        directories::BaseDirs::new()
            .map(|b| b.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dropvoice")
    }

    /// Returns the path to `dropvoice.toml`.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("dropvoice.toml")
    }

    /// Loads the config, creating a default one on first run.
    pub fn load() -> AppResult<Self> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
        let mut config: DropVoiceConfig = toml::from_str(&contents).map_err(|e| {
            AppError::Internal(format!("failed to parse {}: {}", path.display(), e))
        })?;

        migration::migrate_config(&mut config);
        Ok(config)
    }

    /// Persists the config to disk.
    pub fn save(&self) -> AppResult<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
        }
        let contents = toml::to_string_pretty(self)
            .map_err(|e| AppError::Internal(format!("failed to serialize config: {}", e)))?;
        fs::write(&path, contents).map_err(|e| AppError::io(&path, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = DropVoiceConfig::default();
        assert_eq!(config.meta.config_version, 2);
        assert_eq!(config.server.port, 38425);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.injection.max_text_length, 10000);
        assert_eq!(config.security.pairing_code_length, 6);
        assert_eq!(config.security.rate_limit_per_minute, 30);
        assert_eq!(config.devices.max_count, 5);
        assert_eq!(config.app.language, "en");
        assert_eq!(config.app.theme, "system");
        // Device config (spec 11 §7.1).
        assert!(!config.device.device_id.is_empty());
        assert!(!config.device.device_name.is_empty());
        assert!(config.device.pairing_token.is_none());
    }

    #[test]
    fn default_config_round_trips_through_toml() {
        let config = DropVoiceConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: DropVoiceConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.server.port, config.server.port);
        assert_eq!(parsed.app.language, config.app.language);
    }

    #[test]
    fn migration_v0_to_v2() {
        let mut config = DropVoiceConfig::default();
        config.meta.config_version = 0;
        migration::migrate_config(&mut config);
        assert_eq!(config.meta.config_version, 2);
        // device fields auto-populated by serde(default).
        assert!(!config.device.device_id.is_empty());
    }

    #[test]
    fn migration_v1_to_v2() {
        let mut config = DropVoiceConfig::default();
        config.meta.config_version = 1;
        migration::migrate_config(&mut config);
        assert_eq!(config.meta.config_version, 2);
        assert!(!config.device.device_id.is_empty());
    }

    #[test]
    fn migration_v2_is_noop() {
        let mut config = DropVoiceConfig::default();
        config.meta.config_version = 2;
        let before = config.server.port;
        let device_id = config.device.device_id.clone();
        migration::migrate_config(&mut config);
        assert_eq!(config.meta.config_version, 2);
        assert_eq!(config.server.port, before);
        assert_eq!(config.device.device_id, device_id);
    }

    #[test]
    fn unknown_version_resets_to_default() {
        let mut config = DropVoiceConfig::default();
        config.meta.config_version = 999;
        migration::migrate_config(&mut config);
        assert_eq!(config.meta.config_version, 2);
    }
}

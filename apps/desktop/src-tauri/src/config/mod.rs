use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// 配对/信令服务器默认地址（生产 API，PWA 与 API 同源，§10.4）。
///
/// URL 真源是 `config.toml` 的 `network.pairing_server_url`（serde 默认值
/// 引用本常量）；env `PAIRING_SERVER_URL` 仅作开发编排覆盖（.vscode/tasks.json
/// 内置，见 `network::pairing_client::resolve_base_url`）。staging
/// （https://dropvoice.bytehome.fun）是内网 dogfood 场，不作发布默认值。
pub const DEFAULT_PAIRING_SERVER_URL: &str = "https://api.dropvoice.app";

/// Root configuration loaded from `config.toml`
/// (`<config_dir>/dropvoice/config.toml`, Windows:
/// `%APPDATA%\dropvoice\config.toml`).
///
/// 只保留真实生效的字段——每个字段都有消费者（命令 / ConnectionManager /
/// heartbeat / 日志清理）。窗口尺寸与更新器由 `tauri.conf.json` 单独管辖，
/// 不在此重复。无 migration：TOML 解析忽略未知字段，删字段不需要迁移。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DropVoiceConfig {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub injection: InjectionConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub window: WindowConfig,
    /// Per-device identity and pairing state (spec 11 §7.1 / §14).
    #[serde(default)]
    pub device: DeviceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
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
            language: default_language(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

fn default_port() -> u16 {
    38425
}

fn default_max_connections() -> usize {
    10
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 配对/信令服务器 base URL（heartbeat 注册 + webview SSE 共用，
    /// 由 `get_signaling_url` 命令下发）。env `PAIRING_SERVER_URL` 覆盖之。
    #[serde(default = "default_pairing_server_url")]
    pub pairing_server_url: String,
}

fn default_pairing_server_url() -> String {
    DEFAULT_PAIRING_SERVER_URL.to_string()
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            pairing_server_url: default_pairing_server_url(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionConfig {
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    /// 单次注入文本上限（字符数），`inject_text` 命令与注入器双层校验。
    #[serde(default = "default_max_text_length")]
    pub max_text_length: usize,
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
}

fn default_delay_ms() -> u64 {
    10
}

fn default_max_text_length() -> usize {
    crate::text::DEFAULT_MAX_TEXT_LENGTH
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

/// 配对安全参数。配对码固定 6 位数字（webrtc-scan-direct-design §3.2），
/// 不作配置项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 配对码 TTL（分钟），同时是自主轮换周期。
    #[serde(default = "default_pairing_code_expiry")]
    pub pairing_code_expiry_minutes: u64,
}

fn default_pairing_code_expiry() -> u64 {
    5
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            pairing_code_expiry_minutes: default_pairing_code_expiry(),
        }
    }
}

/// 日志保留策略。日志开关/目录不可配（始终开启，目录平台固定，见
/// `telemetry/logging.rs`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_log_retention")]
    pub log_retention_days: u64,
}

fn default_log_retention() -> u64 {
    30
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_retention_days: default_log_retention(),
        }
    }
}

/// 窗口行为。窗口尺寸等由 `tauri.conf.json` 管辖。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: default_minimize_to_tray(),
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
    ///
    /// 兼容旧版 `Vec<String>`（无签发时间）——反序列化时视为"现在签发"，
    /// 获得完整 30 天 TTL 窗口。详见 `deserialize_connected_tokens`。
    #[serde(default, deserialize_with = "deserialize_connected_tokens")]
    pub connected_tokens: Option<Vec<ConnectionTokenRecord>>,
}

/// 一个持久化的连接令牌记录：令牌 + 签发时间（30 天 TTL，`auth::CONNECTION_TOKEN_TTL`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTokenRecord {
    pub token: String,
    pub issued_at: DateTime<Utc>,
}

/// 兼容两种持久化形态：
/// - 旧版（字符串数组）：`["dvct_...", ...]` → 签发时间记为现在（30 天窗口从本轮起算）。
/// - 新版（记录数组）：`[{ token, issued_at }, ...]`。
fn deserialize_connected_tokens<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<ConnectionTokenRecord>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Tokens {
        Legacy(Vec<String>),
        Records(Vec<ConnectionTokenRecord>),
    }

    let opt = Option::<Tokens>::deserialize(deserializer)?;
    Ok(match opt {
        None => None,
        Some(Tokens::Records(records)) => Some(records),
        Some(Tokens::Legacy(tokens)) => Some(
            tokens
                .into_iter()
                .map(|token| ConnectionTokenRecord {
                    token,
                    issued_at: Utc::now(),
                })
                .collect(),
        ),
    })
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

    /// Returns the path to `config.toml`.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
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
        let config: DropVoiceConfig = toml::from_str(&contents).map_err(|e| {
            AppError::Internal(format!("failed to parse {}: {}", path.display(), e))
        })?;
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
        assert_eq!(config.server.port, 38425);
        assert_eq!(config.server.max_connections, 10);
        assert_eq!(
            config.network.pairing_server_url,
            DEFAULT_PAIRING_SERVER_URL
        );
        assert_eq!(config.injection.delay_ms, 10);
        assert_eq!(config.injection.max_text_length, 10_000);
        assert_eq!(config.injection.queue_size, 100);
        assert_eq!(config.security.pairing_code_expiry_minutes, 5);
        assert_eq!(config.telemetry.log_retention_days, 30);
        assert!(config.window.minimize_to_tray);
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
        assert_eq!(
            parsed.injection.max_text_length,
            config.injection.max_text_length
        );
    }

    /// 无 migration 的依据：TOML 解析忽略未知字段——含已删除历史字段
    /// （host / updater / devices 等）的旧文件仍可正常加载。
    #[test]
    fn unknown_and_removed_fields_are_ignored() {
        let legacy = r#"
[app]
version = "0.1.0"
[server]
host = "0.0.0.0"
[injection]
max_text_length = 42
[updater]
endpoint = "https://old.example/manifest.json"
[devices]
max_count = 9
"#;
        let parsed: DropVoiceConfig = toml::from_str(legacy).unwrap();
        assert_eq!(parsed.injection.max_text_length, 42);
        assert_eq!(parsed.server.max_connections, 10);
    }

    /// 旧版 `connected_tokens = ["dvct_..."]`（无签发时间）必须仍可加载，
    /// 并视为"现在签发"（完整 30 天 TTL 窗口，无 migration 也兼容升级）。
    #[test]
    fn legacy_connected_tokens_string_array_loads_as_records() {
        let legacy = r#"
[device]
device_id = "dev-1"
device_name = "PC"
connected_tokens = ["dvct_legacy1", "dvct_legacy2"]
"#;
        let parsed: DropVoiceConfig = toml::from_str(legacy).unwrap();
        let tokens = parsed.device.connected_tokens.expect("tokens present");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, "dvct_legacy1");
        // 签发时间约为现在（30 天窗口内有效）。
        let age = Utc::now().signed_duration_since(tokens[0].issued_at);
        assert!(age.num_seconds() < 5);
    }

    /// 新版 `{token, issued_at}` 记录数组正常往返。
    #[test]
    fn connected_token_records_round_trip() {
        let config = DropVoiceConfig {
            device: DeviceConfig {
                connected_tokens: Some(vec![ConnectionTokenRecord {
                    token: "dvct_rec".into(),
                    issued_at: Utc::now(),
                }]),
                ..DeviceConfig::default()
            },
            ..DropVoiceConfig::default()
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: DropVoiceConfig = toml::from_str(&toml_str).unwrap();
        let tokens = parsed.device.connected_tokens.unwrap();
        assert_eq!(tokens[0].token, "dvct_rec");
    }
}

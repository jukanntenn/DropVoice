//! 配置与时间常量（webrtc-scan-direct-design §4.6）。
//!
//! 常量是权威来源。运行时配置按优先级加载（低→高）：
//! 1. 内置默认值（`Config::default()`）
//! 2. TOML 文件（可选；探测顺序见 [`CONFIG_PATHS`]）：
//!    - 容器内 `/app/config.toml`（docker compose 挂载点：验收挂
//!      `docker/config.acceptance.toml`，自部署挂 `docker/config.toml`
//!      （模板 `docker/config.example.toml`），staging/prod 挂 ansible
//!      渲染的 `config.toml.j2`）
//!    - 开发裸跑（cargo run）`apps/pairing-server/config.local.toml`
//!      （gitignored；模板 `config.local.toml.example`）
//! 3. 环境变量覆盖（`LISTEN_ADDR` / `DATABASE_URL` / `RATE_LIMIT_PER_SEC` /
//!    `CORS_ORIGINS`）——仅临时覆盖的逃生舱，常态不用
//!
//! 调试类开关只走 env、不入 TOML（与 RUST_LOG / LOG_DIR 同类，模板间因此
//! 无 docs 差异）：`ENABLE_DOCS`（Swagger UI，默认关）。
//!
//! 配置文件存在但读取/解析失败时**启动即失败**（返回 Err）——静默回落
//! 默认值会让限流/CORS/数据库路径悄然漂移，排查成本远高于一次明确的启动失败。
use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 配置文件探测路径（按序取第一个**存在**的；都不存在则用内置默认值）。
///
/// 容器路径在前（部署形态的权威挂载点）；dev 路径是编译期绝对路径，
/// 容器构建时该路径在运行时镜像中不存在，探测自然跳过。
fn config_paths() -> [PathBuf; 2] {
    [
        PathBuf::from("/app/config.toml"),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/config.local.toml")),
    ]
}

// ──────────────────────────────────────────────────────────────────────────
// 时间常量（SCREAMING_SNAKE_CASE）
// ──────────────────────────────────────────────────────────────────────────

/// 信令会话 TTL（§4.6：60s，显著大于长轮询 hold 30s + ICE 收集 + 余量，
/// 避免边界竞态——若 TTL ≤ hold，hold 未结束时 cleanup_expired() 可能已回收会话）。
pub const SIGNAL_SESSION_TTL: std::time::Duration = std::time::Duration::from_secs(60);

/// 长轮询 hold 时长（§4.1：最多 30s，answer 就绪立即返回，无则 204 超时）。
pub const LONG_POLL_HOLD: std::time::Duration = std::time::Duration::from_secs(30);

/// SSE 心跳周期（§4.6：每 15s 发具名 ping 事件，防 CF 125s Proxy Read Timeout）。
pub const SSE_PING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

/// SSE 连接最大生命周期（240s）。中间层（内网穿透 openresty 实测 300s、Cloudflare
/// 125s）有连接时长上限，掐断后客户端可能不重连（实测）。服务端在此上限前
/// **优雅关闭**流 + 发 `retry: 1000`，EventSource 自动重连，连接永不过期。
pub const SSE_MAX_LIFETIME: std::time::Duration = std::time::Duration::from_secs(240);

/// 每 device 并发信令会话软上限（§4.6：10，防滥用）。
pub const MAX_SESSIONS_PER_DEVICE: usize = 10;

/// SDP 载荷上限（§4.6：64KB，校验拒绝防滥用）。
pub const SDP_MAX_BYTES: usize = 64 * 1024;

/// 配对 Token 有效期（24 小时）。设备最长离线仍能复用旧 token 的语义。
pub const DEVICE_TOKEN_TTL: chrono::Duration = chrono::Duration::hours(24);

/// 桌面端状态上报周期（5 分钟）。服务端用 2× 此值作为"离线"阈值。
pub const DEVICE_STATUS_INTERVAL: chrono::Duration = chrono::Duration::minutes(5);

/// 速率限制：每 IP 每秒最多 1 个请求。
pub const RATE_LIMIT_PER_SEC: u32 = 1;

/// SQLite 忙等待（50ms）。
pub const SQLITE_BUSY_TIMEOUT_MS: u64 = 50;

/// 后台清理周期（1 分钟）：信令会话 GC + 离线标记 + 缓存清理。
pub const CLEANUP_INTERVAL_SECS: u64 = 60;

/// 批量写入周期（1 秒）。
pub const BATCH_WRITE_INTERVAL_SECS: u64 = 1;

/// 配对 token 长度（64 字符）。
pub const PAIRING_TOKEN_LEN: usize = 64;

// ──────────────────────────────────────────────────────────────────────────
// 运行时配置
// ──────────────────────────────────────────────────────────────────────────

/// 服务端运行时配置。
///
/// 字段与 TOML 文件 / 环境变量一一对应（见 `Config::load`）。
/// 所有字段都有内置默认值（`Default`），TOML 文件可省略任意字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// axum 监听地址。默认 `127.0.0.1:38424`（仅回环，经 Caddy 反代）。
    pub listen_addr: String,
    /// SQLite 数据库路径。
    pub database_url: String,
    /// 速率限制（每 IP 每秒请求数）。
    pub rate_limit_per_sec: u32,
    /// CORS 允许的 origin 列表（§10.5：桌面 webview 跨端口直连）。
    /// 空列表表示不启用 CORS（prod 同域部署不触发）。TOML 用数组，
    /// env `CORS_ORIGINS` 用逗号分隔字符串。
    pub cors_origins: Vec<String>,
}

/// 是否挂载 /docs Swagger UI（仅 env `ENABLE_DOCS=1|true`，默认关闭）。
///
/// 调试开关与 RUST_LOG 同类，不入 TOML——各环境模板间不再有 docs 差异。
pub fn enable_docs() -> bool {
    env::var("ENABLE_DOCS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

impl Config {
    /// 按优先级加载配置：内置默认值 → TOML 文件（探测见 [`config_paths`]）→ env 覆盖。
    ///
    /// TOML 中缺失的字段回落内置默认值；env 仅当变量**存在**时覆盖（与
    /// "配置尽量放文件、env 只在非常必要时覆盖"的约定一致）。
    /// TOML 文件存在但读取/解析失败 → Err（启动即失败，见模块文档）。
    pub fn load() -> Result<Self, String> {
        let mut cfg = Config::default();

        let existing = config_paths().into_iter().find(|p| p.exists());
        if let Some(path) = existing {
            let text = std::fs::read_to_string(&path).map_err(|e| {
                format!(
                    "failed to read config {}: {e} (mounted a missing file? docker would have \
                     created a directory — copy the example first, e.g. \
                     `cp config.local.toml.example config.local.toml`)",
                    path.display()
                )
            })?;
            let file_cfg: Config = toml::from_str(&text)
                .map_err(|e| format!("invalid TOML in config {}: {e}", path.display()))?;
            tracing::info!(path = %path.display(), "config loaded from toml");
            cfg = file_cfg;
        } else {
            tracing::info!(
                "no config file found (checked /app/config.toml and config.local.toml); \
                 using built-in defaults"
            );
        }

        if let Ok(v) = env::var("LISTEN_ADDR") {
            cfg.listen_addr = v;
        }
        if let Ok(v) = env::var("DATABASE_URL") {
            cfg.database_url = v;
        }
        if let Ok(v) = env::var("RATE_LIMIT_PER_SEC") {
            match v.parse() {
                Ok(n) => cfg.rate_limit_per_sec = n,
                Err(_) => {
                    tracing::warn!(value = %v, "invalid RATE_LIMIT_PER_SEC, keeping configured value")
                }
            }
        }
        if let Ok(v) = env::var("CORS_ORIGINS") {
            cfg.cors_origins = if v.trim().is_empty() {
                Vec::new()
            } else {
                v.split(',').map(|o| o.trim().to_string()).collect()
            };
        }
        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:38424".into(),
            // 默认使用临时文件路径；生产/测试应通过 DATABASE_URL 注入。
            database_url: {
                let mut p = default_data_dir();
                p.push("pairing.db");
                format!("sqlite://{}?mode=rwc", p.display())
            },
            rate_limit_per_sec: RATE_LIMIT_PER_SEC,
            cors_origins: Vec::new(),
        }
    }
}

/// 默认数据目录（应用数据目录）。
fn default_data_dir() -> PathBuf {
    let dirs: Option<directories::ProjectDirs> =
        directories::ProjectDirs::from("app", "dropvoice", "dropvoice");
    match dirs {
        Some(p) => p.data_dir().to_path_buf(),
        None => PathBuf::from("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_constants_match_spec() {
        assert_eq!(SIGNAL_SESSION_TTL, std::time::Duration::from_secs(60));
        assert_eq!(LONG_POLL_HOLD, std::time::Duration::from_secs(30));
        assert_eq!(SSE_PING_INTERVAL, std::time::Duration::from_secs(15));
        assert_eq!(SSE_MAX_LIFETIME, std::time::Duration::from_secs(240));
        assert_eq!(MAX_SESSIONS_PER_DEVICE, 10);
        assert_eq!(SDP_MAX_BYTES, 64 * 1024);
        assert_eq!(DEVICE_TOKEN_TTL.num_hours(), 24);
        assert_eq!(PAIRING_TOKEN_LEN, 64);
        assert_eq!(RATE_LIMIT_PER_SEC, 1);
        assert_eq!(SQLITE_BUSY_TIMEOUT_MS, 50);
        assert_eq!(CLEANUP_INTERVAL_SECS, 60);
        assert_eq!(BATCH_WRITE_INTERVAL_SECS, 1);
    }

    #[test]
    fn default_listen_addr_is_loopback_38424() {
        let c = Config::default();
        assert!(c.listen_addr.contains("38424"));
        assert!(c.listen_addr.starts_with("127.0.0.1"));
    }

    /// UT-config-03：rate_limit_per_sec 默认值。
    #[test]
    fn rate_limit_per_sec_default() {
        let c = Config::default();
        assert_eq!(c.rate_limit_per_sec, RATE_LIMIT_PER_SEC);
    }
}

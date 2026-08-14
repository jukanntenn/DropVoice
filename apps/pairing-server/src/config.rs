//! 配置与时间常量（webrtc-scan-direct-design §4.6）。
//!
//! 常量是权威来源。运行时配置按优先级加载（低→高）：
//! 1. 内置默认值（`Config::default()`）
//! 2. 容器内固定路径 `/app/config.toml`（可选；docker-compose.local 挂载
//!    `config.local.toml` 到此路径，字段缺失时回落默认值）
//! 3. 环境变量覆盖（`LISTEN_ADDR` / `DATABASE_URL` / `RATE_LIMIT_PER_SEC` /
//!    `ENABLE_DOCS` / `CORS_ORIGINS`）——仅当需要临时覆盖时使用
//!
//! 生产 / staging 由 docker-compose 提供，目前仍走 env（`Config::load()`
//! 自动兼容；将来可整体切换为 TOML）。
use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 容器内默认配置文件路径（docker-compose.local 挂载于此）。
/// 裸跑（cargo run）时该路径不存在，自动跳过。
const DEFAULT_CONFIG_PATH: &str = "/app/config.toml";

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
    /// 是否挂载 /docs Swagger UI 交互文档。默认关闭（生产经 Caddy 反代时不暴露公网）。
    pub enable_docs: bool,
    /// CORS 允许的 origin 列表（§10.5：桌面 webview 跨端口直连）。
    /// 空列表表示不启用 CORS（prod 同域部署不触发）。TOML 用数组，
    /// env `CORS_ORIGINS` 用逗号分隔字符串。
    pub cors_origins: Vec<String>,
}

impl Config {
    /// 按优先级加载配置：内置默认值 → `/app/config.local.toml`（可选）→ env 覆盖。
    ///
    /// TOML 中缺失的字段回落内置默认值；env 仅当变量**存在**时覆盖（与
    /// "配置尽量放文件、env 只在非常必要时覆盖"的约定一致）。
    pub fn load() -> Self {
        let mut cfg = Config::default();

        let path = PathBuf::from(DEFAULT_CONFIG_PATH);
        if path.exists() {
            match std::fs::read_to_string(&path).and_then(|text| {
                toml::from_str::<Config>(&text)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }) {
                Ok(file_cfg) => {
                    tracing::info!(path = %path.display(), "config loaded from toml");
                    cfg = file_cfg;
                }
                Err(e) => tracing::warn!(
                    error = %e,
                    path = %path.display(),
                    "invalid config toml, falling back to defaults"
                ),
            }
        }

        if let Ok(v) = env::var("LISTEN_ADDR") {
            cfg.listen_addr = v;
        }
        if let Ok(v) = env::var("DATABASE_URL") {
            cfg.database_url = v;
        }
        if let Ok(v) = env::var("RATE_LIMIT_PER_SEC") {
            if let Ok(n) = v.parse() {
                cfg.rate_limit_per_sec = n;
            }
        }
        if let Ok(v) = env::var("ENABLE_DOCS") {
            cfg.enable_docs = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = env::var("CORS_ORIGINS") {
            cfg.cors_origins = if v.trim().is_empty() {
                Vec::new()
            } else {
                v.split(',').map(|o| o.trim().to_string()).collect()
            };
        }
        cfg
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
            enable_docs: false,
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

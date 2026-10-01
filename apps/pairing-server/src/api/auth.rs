//! 认证提取器（Bearer token）+ 速率限制。
//!
//! 认证流程：从 `Authorization: Bearer <token>` 提取 token，优先查 token 缓存
//! （键为明文 token），回查 DB（按哈希比对，见 `device_repo`）。失败返回
//! `TokenInvalid`。
//!
//! 凭据模型不变量：明文 token 只存在于颁发瞬间与内存缓存；DB 只存
//! [`token_hash`]。SSE 订阅不再使用长效 query token（日志泄露面），改用
//! 一次性票据（`signaling_handlers::subscribe_events`）。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::error::AppError;

/// 认证后的设备 id。
pub struct AuthenticatedDevice(pub Uuid);

impl<S> FromRequestParts<S> for AuthenticatedDevice
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // 从 Authorization 头提取 Bearer token。
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.trim())
            .ok_or(AppError::TokenInvalid)?;

        if token.is_empty() {
            return Err(AppError::TokenInvalid);
        }

        verify_token(&app_state, token)
            .await
            .map(AuthenticatedDevice)
    }
}

/// token 校验公共逻辑：cache → DB（哈希比对）。
async fn verify_token(app_state: &AppState, token: &str) -> Result<Uuid, AppError> {
    // 优先查缓存。
    if let Some(id) = app_state.cache.get_token(token).await {
        return Ok(id);
    }
    // 回查 DB。
    match crate::store::device_repo::find_id_by_token(&app_state.pool, token).await? {
        Some(id) => {
            app_state.cache.put_token(token, id).await;
            Ok(id)
        }
        None => Err(AppError::TokenInvalid),
    }
}

/// token 的 SHA-256 hex（落库形态）。明文 token 只在颁发瞬间与内存缓存存在。
pub fn token_hash(token: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// 每 IP 速率限制器（`RATE_LIMIT_PER_SEC`，sliding window，进程内实现）。
///
/// 内存有界（防伪造 IP 撑键集合）：周期清扫空桶；键数硬上限触发整体重置
/// （宁可短暂放行也不 OOM——外层还有 Cloudflare 限流规则兜底）。
#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    inner: Arc<tokio::sync::RwLock<Buckets>>,
}

#[derive(Debug, Default)]
struct Buckets {
    map: HashMap<String, Vec<Instant>>,
    last_sweep: Option<Instant>,
}

/// 空桶清扫最小间隔（与请求路径解耦，避免每请求 O(n) 扫描）。
const SWEEP_INTERVAL: Duration = Duration::from_secs(30);

/// 键数硬上限（伪造 IP 的无界增长封顶；超出即重置并告警）。
const MAX_BUCKETS: usize = 50_000;

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查 `client_ip` 是否允许本次请求。允许→true，超限→false。
    pub async fn check(&self, client_ip: &str, limit_per_sec: u32) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(1);
        let mut state = self.inner.write().await;

        let allowed = {
            let bucket = state.map.entry(client_ip.to_string()).or_default();
            bucket.retain(|t| now.duration_since(*t) < window);
            if bucket.len() >= limit_per_sec as usize {
                false
            } else {
                bucket.push(now);
                true
            }
        };

        // 有界性维护：超上限立即清扫；否则按最小间隔清扫。
        let overdue = state
            .last_sweep
            .is_none_or(|t| now.duration_since(t) >= SWEEP_INTERVAL);
        if allowed && (overdue || state.map.len() > MAX_BUCKETS) {
            state.last_sweep = Some(now);
            state.map.retain(|_, v| !v.is_empty());
            if state.map.len() > MAX_BUCKETS {
                tracing::warn!(
                    keys = state.map.len(),
                    limit = MAX_BUCKETS,
                    "rate limiter bucket overflow; resetting all buckets"
                );
                state.map.clear();
            }
        }

        allowed
    }

    /// 当前键数（测试/诊断用）。
    #[cfg(test)]
    pub async fn bucket_count(&self) -> usize {
        self.inner.read().await.map.len()
    }
}

/// 从请求头和连接地址中提取 client IP。
/// 取 `X-Forwarded-For` 第一个（Caddy 已按信任链重写为真实客户端 IP 单值），
/// 否则连接地址。
pub fn extract_client_ip(
    headers: &axum::http::HeaderMap,
    connect_addr: &Option<SocketAddr>,
) -> String {
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(s) = xff.to_str() {
            if let Some(first) = s.split(',').next() {
                let trimmed = first.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    connect_addr
        .map(|ci| ci.ip().to_string())
        .unwrap_or_default()
}

/// 从 Parts 中提取 client IP（中间件 / handler 内使用）。
pub fn extract_client_ip_from_parts(parts: &Parts) -> String {
    let connect_addr = parts
        .extensions
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);
    extract_client_ip(&parts.headers, &connect_addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use crate::{clock, observability, store};
    use std::sync::Arc;

    /// UT-RateLimiter-01：窗口内允许 N 次。
    #[tokio::test]
    async fn rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("1.2.3.4", 1).await);
    }

    /// UT-RateLimiter-02：超限拒绝。
    #[tokio::test]
    async fn rate_limiter_rejects_over_limit() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("1.2.3.4", 1).await);
        assert!(!limiter.check("1.2.3.4", 1).await);
    }

    /// UT-RateLimiter-03：滑动窗口过期。
    #[tokio::test]
    async fn rate_limiter_window_expires() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("1.2.3.4", 1).await);
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(limiter.check("1.2.3.4", 1).await);
    }

    /// UT-RateLimiter-04：不同 IP 独立计数。
    #[tokio::test]
    async fn rate_limiter_independent_per_ip() {
        let limiter = RateLimiter::new();
        assert!(limiter.check("1.2.3.4", 1).await);
        assert!(!limiter.check("1.2.3.4", 1).await);
        assert!(limiter.check("5.6.7.8", 1).await);
    }

    /// UT-RateLimiter-05：高限流值允许多次请求。
    #[tokio::test]
    async fn rate_limiter_high_limit() {
        let limiter = RateLimiter::new();
        for _ in 0..100 {
            assert!(limiter.check("1.2.3.4", 100).await);
        }
    }

    /// UT-RateLimiter-06：limit=0 拒绝所有。
    #[tokio::test]
    async fn rate_limiter_zero_limit() {
        let limiter = RateLimiter::new();
        assert!(!limiter.check("1.2.3.4", 0).await);
    }

    /// UT-RateLimiter-07：伪造海量 IP 时键数有界（溢出重置）。
    #[tokio::test]
    async fn rate_limiter_bounded_under_ip_flooding() {
        let limiter = RateLimiter::new();
        for i in 0..(MAX_BUCKETS + 100) {
            let ip = format!("10.{}.{}.{}", (i >> 16) % 256, (i >> 8) % 256, i % 256);
            limiter.check(&ip, 1).await;
        }
        assert!(
            limiter.bucket_count().await <= MAX_BUCKETS,
            "bucket count must stay bounded"
        );
    }

    /// UT-token-hash-01：确定性与 hex 形态。
    #[test]
    fn token_hash_is_sha256_hex() {
        let h = token_hash("abc");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        // SHA-256("abc") 的知名向量。
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// UT-extract_ip-01：XFF 单段。
    #[test]
    fn extract_ip_xff_single() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-02：XFF 多段。
    #[test]
    fn extract_ip_xff_multiple() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-03：XFF 含空白。
    #[test]
    fn extract_ip_xff_with_whitespace() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", " 1.2.3.4 , 5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-04：无 XFF 回退 ConnectInfo。
    #[test]
    fn extract_ip_fallback_to_connect_info() {
        let headers = axum::http::HeaderMap::new();
        let addr: SocketAddr = "10.0.0.1:12345".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, &Some(addr)), "10.0.0.1");
    }

    /// UT-extract_ip-05：无 XFF 无 ConnectInfo。
    #[test]
    fn extract_ip_empty_when_no_info() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(extract_client_ip(&headers, &None), "");
    }

    /// UT-extract_ip-06：XFF 为空字符串时回退 ConnectInfo。
    #[test]
    fn extract_ip_xff_empty_fallback() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "".parse().unwrap());
        let addr: SocketAddr = "10.0.0.1:12345".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, &Some(addr)), "10.0.0.1");
    }

    /// UT-extract_ip-07：extract_client_ip_from_parts 无 ConnectInfo。
    #[test]
    fn extract_ip_from_parts_no_connect_info() {
        let req = axum::http::Request::builder()
            .header("x-forwarded-for", "1.2.3.4")
            .body(axum::body::Body::empty())
            .unwrap();
        let (parts, _) = req.into_parts();
        assert_eq!(extract_client_ip_from_parts(&parts), "1.2.3.4");
    }

    /// UT-extract_ip-08：extract_client_ip_from_parts 有 ConnectInfo。
    #[test]
    fn extract_ip_from_parts_with_connect_info() {
        let mut req = axum::http::Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let addr = axum::extract::ConnectInfo::<SocketAddr>("10.0.0.1:12345".parse().unwrap());
        req.extensions_mut().insert(addr);
        let (parts, _) = req.into_parts();
        assert_eq!(extract_client_ip_from_parts(&parts), "10.0.0.1");
    }

    /// 测试辅助：构造带 DB 的 AppState。
    async fn test_state_with_db() -> (AppState, sqlx::SqlitePool) {
        let tmp = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", tmp.path().join("t.db").display());
        let pool = store::open_pool(&url).await.unwrap();
        let clk: Arc<dyn clock::Clock> = Arc::new(clock::SystemClock);
        let metrics = Arc::new(observability::Metrics::default());
        // 保持 tempdir 存活：测试内 pool 引用同一 db 文件，tempdir drop 会删文件，
        // 但 pool 已打开连接，SQLite 连接保持文件有效。这里不持有 tempdir（测试短促）。
        std::mem::forget(tmp);
        let state = AppState::new(pool.clone(), metrics, clk, 1000);
        (state, pool)
    }

    /// 构造一个已注册设备（返回 device_id 与明文 token）。
    async fn seeded_device(pool: &sqlx::SqlitePool) -> (Uuid, String) {
        let device_id = Uuid::new_v4();
        let req = crate::domain::DeviceRegisterRequest {
            device_id,
            platform: crate::domain::Platform::Desktop,
            device_name: None,
            address: crate::domain::DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        let clk = clock::SystemClock;
        crate::store::device_repo::upsert(pool, &req, None, "seed-token", &clk)
            .await
            .unwrap();
        (device_id, "seed-token".into())
    }

    /// UT-AuthenticatedDevice-01：无 Authorization 头 → TokenInvalid。
    #[tokio::test]
    async fn authenticated_device_no_auth_header() {
        let (state, _pool) = test_state_with_db().await;

        let req = axum::http::Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(matches!(result, Err(crate::error::AppError::TokenInvalid)));
    }

    /// UT-AuthenticatedDevice-02：DB 未命中（哈希比对）→ TokenInvalid。
    #[tokio::test]
    async fn authenticated_device_db_miss() {
        let (state, _pool) = test_state_with_db().await;

        let req = axum::http::Request::builder()
            .header(
                "Authorization",
                "Bearer fake-token-64-characters-long-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            )
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(matches!(result, Err(crate::error::AppError::TokenInvalid)));
    }

    /// UT-AuthenticatedDevice-03：DB 命中 → Ok + 写缓存。
    #[tokio::test]
    async fn authenticated_device_db_hit() {
        let (state, pool) = test_state_with_db().await;
        let (device_id, token) = seeded_device(&pool).await;

        let req = axum::http::Request::builder()
            .header("Authorization", format!("Bearer {token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, device_id);

        // 验证 token 已写入缓存。
        assert_eq!(state.cache.get_token(&token).await, Some(device_id));
    }

    /// UT-AuthenticatedDevice-04：缓存命中 → Ok（不查 DB）。
    #[tokio::test]
    async fn authenticated_device_cache_hit() {
        let (state, _pool) = test_state_with_db().await;
        let device_id = Uuid::new_v4();
        state.cache.put_token("cached-token", device_id).await;

        let req = axum::http::Request::builder()
            .header("Authorization", "Bearer cached-token")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, device_id);
    }
}

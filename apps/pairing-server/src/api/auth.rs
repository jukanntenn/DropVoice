//! 认证提取器（Bearer token）+ 速率限制（spec 11 §14.1）。
//!
//! 认证流程：从 `Authorization: Bearer <token>` 提取 token，优先查 token 缓存，
//! 回查 DB。失败返回 `TokenInvalid`。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::HeaderMap;
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

/// SSE 端点用的 query 版认证 extractor（§4.8）。
///
/// EventSource 不支持自定义 header，pairing_token 通过 `?token=` 传递。
/// 复用与 `AuthenticatedDevice` 相同的 token 校验逻辑（cache → DB），只换提取源。
/// 威胁模型与 Bearer 不同：SSE 订阅是"旁听某设备的所有 offer 流"，需 query token 鉴权。
pub struct AuthenticatedDeviceFromQuery(pub Uuid);

impl<S> FromRequestParts<S> for AuthenticatedDeviceFromQuery
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // 从 query string 提取 token。
        let query =
            axum::extract::Query::<std::collections::HashMap<String, String>>::from_request_parts(
                parts, state,
            )
            .await
            .map_err(|_| AppError::TokenInvalid)?;

        let token = query
            .0
            .get("token")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let token = token.ok_or(AppError::TokenInvalid)?;

        verify_token(&app_state, token)
            .await
            .map(AuthenticatedDeviceFromQuery)
    }
}

/// token 校验公共逻辑：cache → DB（供 Bearer / query 两个 extractor 共用）。
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

/// 简单的每 IP 速率限制器（`RATE_LIMIT_PER_SEC`，sliding window）。
/// 生产可替换为 Redis-backed 实现；v1.1.0 单机内存即可。
#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    inner: Arc<tokio::sync::RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查 `client_ip` 是否允许本次请求。允许→true，超限→false。
    pub async fn check(&self, client_ip: &str, limit_per_sec: u32) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(1);
        let mut map = self.inner.write().await;
        let bucket = map.entry(client_ip.to_string()).or_default();
        bucket.retain(|t| now.duration_since(*t) < window);
        if bucket.len() >= limit_per_sec as usize {
            return false;
        }
        bucket.push(now);
        true
    }
}

/// 从请求头和连接地址中提取 client IP。
/// 取 `X-Forwarded-For` 第一个（Caddy/Cloudflare 已设置），否则连接地址。
pub fn extract_client_ip(headers: &HeaderMap, connect_addr: &Option<SocketAddr>) -> String {
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

/// 从 Parts 中提取 client IP（兼容旧签名，供 handler 内使用）。
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

    /// UT-extract_ip-01：XFF 单段。
    #[test]
    fn extract_ip_xff_single() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-02：XFF 多段。
    #[test]
    fn extract_ip_xff_multiple() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4, 5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-03：XFF 含空白。
    #[test]
    fn extract_ip_xff_with_whitespace() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", " 1.2.3.4 , 5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers, &None), "1.2.3.4");
    }

    /// UT-extract_ip-04：无 XFF 回退 ConnectInfo。
    #[test]
    fn extract_ip_fallback_to_connect_info() {
        let headers = HeaderMap::new();
        let addr: SocketAddr = "10.0.0.1:12345".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, &Some(addr)), "10.0.0.1");
    }

    /// UT-extract_ip-05：无 XFF 无 ConnectInfo。
    #[test]
    fn extract_ip_empty_when_no_info() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers, &None), "");
    }

    /// UT-extract_ip-06：XFF 为空字符串时回退 ConnectInfo。
    #[test]
    fn extract_ip_xff_empty_fallback() {
        let mut headers = HeaderMap::new();
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

    /// 测试辅助：构造带 DB 的 AppState。
    async fn test_state_with_db() -> (
        AppState,
        sqlx::SqlitePool,
        std::sync::Arc<dyn crate::clock::Clock>,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", tmp.path().join("t.db").display());
        let pool = crate::store::open_pool(&url).await.unwrap();
        let clk: std::sync::Arc<dyn crate::clock::Clock> =
            std::sync::Arc::new(crate::clock::SystemClock);
        let metrics = std::sync::Arc::new(crate::observability::Metrics::default());
        // 保持 tempdir 存活：测试内 pool 引用同一 db 文件，tempdir drop 会删文件，
        // 但 pool 已打开连接，SQLite 连接保持文件有效。这里不持有 tempdir（测试短促）。
        std::mem::forget(tmp);
        let state = AppState::new(pool.clone(), metrics, clk.clone(), 1000);
        (state, pool, clk)
    }

    /// UT-AuthenticatedDevice-01：无 Authorization 头 → TokenInvalid。
    #[tokio::test]
    async fn authenticated_device_no_auth_header() {
        let (state, _pool, _clk) = test_state_with_db().await;

        let req = axum::http::Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(matches!(result, Err(crate::error::AppError::TokenInvalid)));
    }

    /// UT-AuthenticatedDevice-02：DB 未命中 → TokenInvalid。
    #[tokio::test]
    async fn authenticated_device_db_miss() {
        let (state, _pool, _clk) = test_state_with_db().await;

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
        let (state, pool, clk) = test_state_with_db().await;

        // 先注册设备。
        let device_id = uuid::Uuid::new_v4();
        let req = crate::domain::DeviceRegisterRequest {
            device_id,
            platform: crate::domain::Platform::Desktop,
            device_name: None,
            address: crate::domain::DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        crate::store::device_repo::upsert(&pool, &req, "my-token", clk.as_ref())
            .await
            .unwrap();

        let http_req = axum::http::Request::builder()
            .header("Authorization", "Bearer my-token")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = http_req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, device_id);

        // 验证 token 已写入缓存。
        assert_eq!(state.cache.get_token("my-token").await, Some(device_id));
    }

    /// UT-AuthenticatedDevice-04：缓存命中 → Ok（不查 DB）。
    #[tokio::test]
    async fn authenticated_device_cache_hit() {
        let (state, _pool, _clk) = test_state_with_db().await;
        let device_id = uuid::Uuid::new_v4();
        state.cache.put_token("cached-token", device_id).await;

        let http_req = axum::http::Request::builder()
            .header("Authorization", "Bearer cached-token")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = http_req.into_parts();
        let result = AuthenticatedDevice::from_request_parts(&mut parts, &state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, device_id);
    }

    // ── query 版认证 extractor（SSE 用，§4.8）──

    /// UT-AuthenticatedDeviceFromQuery-01：无 token query → TokenInvalid。
    #[tokio::test]
    async fn query_auth_no_token() {
        let (state, _pool, _clk) = test_state_with_db().await;

        let req = axum::http::Request::builder()
            .uri("/events")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDeviceFromQuery::from_request_parts(&mut parts, &state).await;
        assert!(matches!(result, Err(crate::error::AppError::TokenInvalid)));
    }

    /// UT-AuthenticatedDeviceFromQuery-02：有效 token query → Ok。
    #[tokio::test]
    async fn query_auth_valid_token() {
        let (state, pool, clk) = test_state_with_db().await;
        let device_id = uuid::Uuid::new_v4();
        let req = crate::domain::DeviceRegisterRequest {
            device_id,
            platform: crate::domain::Platform::Desktop,
            device_name: None,
            address: crate::domain::DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        crate::store::device_repo::upsert(&pool, &req, "sse-token", clk.as_ref())
            .await
            .unwrap();

        let req = axum::http::Request::builder()
            .uri("/events?token=sse-token")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDeviceFromQuery::from_request_parts(&mut parts, &state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, device_id);
    }

    /// UT-AuthenticatedDeviceFromQuery-03：无效 token query → TokenInvalid。
    #[tokio::test]
    async fn query_auth_invalid_token() {
        let (state, _pool, _clk) = test_state_with_db().await;

        let req = axum::http::Request::builder()
            .uri("/events?token=invalid")
            .body(axum::body::Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let result = AuthenticatedDeviceFromQuery::from_request_parts(&mut parts, &state).await;
        assert!(matches!(result, Err(crate::error::AppError::TokenInvalid)));
    }
}

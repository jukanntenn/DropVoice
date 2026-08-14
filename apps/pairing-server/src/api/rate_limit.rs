//! 速率限制中间件（spec 11 §14.1, §4 `RATE_LIMIT`）。
//!
//! Pre-req-2：RateLimiter 已定义于 auth.rs，本模块将其接入 axum 路由层。
//! 每个请求经过中间件时检查 client IP，超限返回 429 RATE_LIMITED。

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::api::auth::extract_client_ip;
use crate::api::state::AppState;
use crate::error::AppError;

/// 速率限制中间件。从请求中提取 client IP，检查 RateLimiter。
pub async fn rate_limit_layer(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(req.headers(), &Some(addr));
    if !state
        .rate_limiter
        .check(&ip, state.rate_limit_per_sec)
        .await
    {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(req).await)
}

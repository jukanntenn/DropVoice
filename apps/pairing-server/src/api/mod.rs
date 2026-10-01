//! HTTP API 路由层。
//!
//! 路由总览（webrtc-scan-direct-design §4.1）：
//!
//! 业务路由（rate-limited，`/api` 前缀）：
//! | Method | Path | Auth | Handler |
//! |--------|------|------|---------|
//! | POST | /api/devices | 可选 Bearer（复用/轮换需凭据） | `devices::register` |
//! | PUT  | /api/devices/{id}/status | Bearer | `devices::report_status` |
//! | POST | /api/devices/{id}/webrtc/offer | body code/token | `signaling_handlers::create_offer` |
//!
//! 信令路由（部分 rate-exempt，§4.8 独立 Router merge）：
//! | Method | Path | Auth | Handler | Rate-limit |
//! |--------|------|------|---------|------------|
//! | GET  | /api/devices/{id}/webrtc/answer/{session_id} | session_id（凭据） | `signaling_handlers::poll_answer` | 豁免（并发帽） |
//! | POST | /api/devices/{id}/webrtc/subscribe | Bearer | `signaling_handlers::subscribe_ticket` | 豁免（桌面低频） |
//! | GET  | /api/devices/{id}/webrtc/events | query ticket（一次性） | `signaling_handlers::subscribe_events` | 豁免（并发帽） |
//! | POST | /api/devices/{id}/webrtc/answer | Bearer | `signaling_handlers::submit_answer` | 豁免（桌面低频） |
//!
//! 健康检查（无 `/api` 前缀）：
//! | GET | /health | 无 | `health` |

pub mod auth;
pub mod devices;
pub mod error;
pub mod rate_limit;
pub mod signaling;
pub mod signaling_handlers;
pub mod state;

use std::time::Duration;

use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// 构建限速业务路由（`/api` 前缀，rate-limited）。
///
/// rate_limit 通过 `route_layer` 作用于调用时已注册的路由（path_router.rs:282
/// 确认 route_layer 在空路由时 panic）。POST offer 保持限速（配对是低频操作）。
fn build_rate_limited_router(state: state::AppState) -> Router {
    Router::new()
        .route("/api/devices", post(devices::register))
        .route(
            "/api/devices/{device_id}/status",
            put(devices::report_status),
        )
        .route(
            "/api/devices/{device_id}/webrtc/offer",
            post(signaling_handlers::create_offer),
        )
        .route("/health", get(health))
        .route_layer(from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_layer,
        ))
        .with_state(state)
}

/// 构建合并的应用路由（限速 + 豁免），供 e2e 测试复用。
/// 不含 CORS / TraceLayer / docs（测试关注路由语义）。
pub fn build_app_router(state: state::AppState) -> Router {
    Router::new()
        .merge(build_rate_limited_router(state.clone()))
        .merge(build_exempt_router(state))
}
///
/// SSE 长连接 + answer 长轮询 + 票据/answer 回填豁免限速：
/// - SSE/长轮询是长连接，限速 1 req/s/IP 会让桌面心跳式重连被判 429；
/// - 票据换取与 answer 回填是桌面低频操作（每次 SSE 重建/配对各 1 次）。
/// - 豁免端点各自有并发帽兜底（config::MAX_CONCURRENT_*），不构成无界资源占用。
fn build_exempt_router(state: state::AppState) -> Router {
    Router::new()
        .route(
            "/api/devices/{device_id}/webrtc/answer/{session_id}",
            get(signaling_handlers::poll_answer),
        )
        .route(
            "/api/devices/{device_id}/webrtc/subscribe",
            post(signaling_handlers::subscribe_ticket),
        )
        .route(
            "/api/devices/{device_id}/webrtc/events",
            get(signaling_handlers::subscribe_events),
        )
        .route(
            "/api/devices/{device_id}/webrtc/answer",
            post(signaling_handlers::submit_answer),
        )
        .with_state(state)
}

/// `GET /health` —— 存活探针（部署探活）。无 `/api` 前缀。
///
/// 返回服务状态、构建版本与当前镜像期望的数据库 schema 版本
/// （`store::MIGRATOR.version()`）。部署冒烟用它断言「迁移已随镜像应用」，
/// 并比对 `version`（镜像构建时烘焙的 `APP_VERSION`，即 `git describe`）
/// 抓住「健康但仍在跑旧镜像」——部署/回滚前后版本可对比。
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "getHealth",
    summary = "存活探针",
    description = "部署用存活探针（Caddy/Cloudflare 健康检查）。返回服务状态、构建版本（APP_VERSION，git describe）与当前镜像期望的数据库 schema 版本。",
    responses(
        (status = 200, description = "服务存活", body = HealthResponse, content_type = "application/json")
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        schema_version: crate::store::MIGRATOR
            .iter()
            .map(|m| m.version)
            .max()
            .unwrap_or_default(),
        version: std::env::var("APP_VERSION").unwrap_or_else(|_| "dev".to_string()),
    })
}

/// `GET /health` 响应体。
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub schema_version: i64,
    /// 构建版本（镜像内 `APP_VERSION` = `git describe`；本地 dev 运行为 "dev"）。
    pub version: String,
}

/// 应用启动：合并限速/豁免路由 + CORS + docs，绑定监听地址。
pub async fn serve(config: Config, state: state::AppState) -> anyhow::Result<()> {
    let addr = config.listen_addr.clone();
    // §4.8：豁免 Router merge 到限速 Router 之外，不继承 route_layer(rate_limit)。
    // route_layer 是 baked-in 属性——它只作用于调用时已注册的路由，
    // merge 进来的独立 Router 不继承（merge 顺序不重要）。
    let mut app = Router::new()
        .merge(build_rate_limited_router(state.clone()))
        .merge(build_exempt_router(state));

    // §10.5 dev CORS：桌面 webview（localhost:5173）直连 :38424 跨端口。
    // prod 同域部署不触发 CORS，cors_origins 为空时跳过。
    if !config.cors_origins.is_empty() {
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .map(|s| s.parse().expect("valid origin"))
            .collect();
        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ]);
        app = app.layer(cors);
    }

    // access log（TraceLayer 输出 http.request span）。
    // client_ip 来自 XFF 最左段（Caddy 已按信任链重写为真实客户端 IP 单值）——
    // 限流 key 用了 IP，日志必须同步记录，否则封禁排查无从下手。
    app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|req: &axum::http::Request<axum::body::Body>| {
                let connect_addr = req
                    .extensions()
                    .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                    .map(|ci| ci.0);
                let client_ip = auth::extract_client_ip(req.headers(), &connect_addr);
                tracing::info_span!(
                    "http.request",
                    method = req.method().as_str(),
                    path = req.uri().path(),
                    client_ip = %client_ip,
                )
            })
            .on_response(
                |response: &axum::response::Response, latency: Duration, _span: &tracing::Span| {
                    tracing::info!(
                        target: "dropvoice::access",
                        status = response.status().as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "request completed"
                    );
                },
            ),
    );

    if crate::config::enable_docs() {
        app = app.merge(crate::docs::build_docs_router());
    }
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, docs_enabled = crate::config::enable_docs(), "pairing server listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth::RateLimiter;
    use crate::{clock, observability, store};
    use std::sync::Arc;

    async fn test_state() -> state::AppState {
        let tmp = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", tmp.path().join("t.db").display());
        let pool = store::open_pool(&url).await.unwrap();
        let clk: Arc<dyn clock::Clock> = Arc::new(clock::SystemClock);
        let metrics = Arc::new(observability::Metrics::default());
        std::mem::forget(tmp);
        state::AppState::new(pool, metrics, clk, 1000)
    }

    /// 测试 health() handler 直接调用（含 schema_version 上报）。
    #[tokio::test]
    async fn health_returns_ok() {
        let result = health().await;
        assert_eq!(result.status, "ok");
        assert!(result.schema_version > 0);
        assert_eq!(result.version, "dev"); // APP_VERSION unset outside the image
    }

    /// 测试限速 + 豁免 Router 合并后 /health 路由可用。
    #[tokio::test]
    async fn merged_router_health_works() {
        let state = test_state().await;
        let app = Router::new()
            .merge(build_rate_limited_router(state.clone()))
            .merge(build_exempt_router(state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}");

        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .ok();
        });

        let resp = reqwest::get(format!("{base}/health")).await.unwrap();
        assert_eq!(resp.status(), 200);
        let body: HealthResponse = resp.json().await.unwrap();
        assert_eq!(body.status, "ok");
        assert!(body.schema_version > 0);
    }

    /// 测试 serve() 函数绑定失败（端口被占用）。
    #[tokio::test]
    async fn serve_bind_failure() {
        let state = test_state().await;
        // 占用端口。
        let _listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = _listener.local_addr().unwrap().to_string();

        let cfg = Config {
            listen_addr: addr,
            database_url: String::new(),
            rate_limit_per_sec: 1000,
            cors_origins: Vec::new(),
        };
        let result = serve(cfg, state).await;
        assert!(result.is_err());
    }

    /// UT-RateLimiter 保留（auth.rs 已有完整覆盖，此处仅确认 RateLimiter 可构造）。
    #[tokio::test]
    async fn rate_limiter_constructs() {
        let _ = RateLimiter::new();
    }
}

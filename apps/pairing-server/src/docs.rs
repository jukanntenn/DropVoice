//! OpenAPI 3.1 自动化文档（utoipa 编译期生成）。
//!
//! - `ApiDoc`：聚合全部 path/schema/组件，是文档的唯一事实源。
//! - `generate_specs()`：生成 JSON + YAML 文档（由 `gen-openapi` bin 导出到
//!   `docs/openapi.{json,yaml}`；`tests/openapi.rs` 用同函数做漂移检测）。
//! - `build_docs_router()`：Swagger UI 交互文档，仅当 `ENABLE_DOCS=true` 时挂载
//!   （见 `api::serve`）。docs 路由在业务路由之外层 merge，不受每 IP 限流影响。
//!
//! 设计基准：`specs/full/openapi/pairing-server.yaml`（只读设计文档）。生成的
//! 文档为 OpenAPI 3.1.0（utoipa 硬编码；设计文档为 3.0.3，语义等价）。

use axum::Router;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{devices, signaling_handlers};
use crate::domain::{
    DeviceAddress, DeviceRegisterRequest, DeviceRegisterResponse, Platform, StatusReportRequest,
    StatusReportResponse,
};
use crate::error::{ErrorBody, ErrorDetail};
use crate::store::device_repo::DeviceSummary;

/// OpenAPI 文档聚合（info/paths/components/tags）。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "DropVoice Pairing & Signaling Server API",
        version = env!("CARGO_PKG_VERSION"),
        description = "公网 HTTPS 配对 + WebRTC 信令服务器。部署在 VPS 上，经 Cloudflare 回源 \
            宿主共享 Caddy:443（终结 TLS），反代容器内 Caddy:8080，再到 axum 127.0.0.1:38424。\n\n\
            ## 职责边界\n\
            本服务只做**设备注册 + WebRTC 信令转发**，绝不中继文本数据。\n\
            手机扫码后发起 offer，服务器通过 SSE 转发给桌面，桌面回填 answer；\n\
            SDP 交换完成后，手机↔桌面 WebRTC DataChannel P2P 直连，服务器退出。\n\n\
            ## 认证\n\
            - `POST /api/devices`：新建设备无需认证；已存在设备须携带 Bearer（当前 \
            token）方可复用/轮换，否则 401——token 绝不复述给无凭据方。\n\
            - `PUT /api/devices/{id}/status`：Bearer token（注册时颁发的 `pairing_token`）。\n\
            - `POST /api/devices/{id}/webrtc/offer`：body 带 code（首次配对）或 token（重连）。\n\
            - `POST /api/devices/{id}/webrtc/subscribe`：Bearer token → 一次性 SSE 票据。\n\
            - `GET /api/devices/{id}/webrtc/events`（SSE）：query `?ticket=`（一次性票据）。\n\
            - `POST /api/devices/{id}/webrtc/answer`：Bearer token。\n\
            - `GET /api/devices/{id}/webrtc/answer/{session_id}`：session_id 自身即为凭据。",
        contact(name = "DropVoice"),
        license(name = "MIT")
    ),
    servers(
        (url = "https://ps.dropvoice.online", description = "生产环境（Cloudflare → 宿主 Caddy:443 → 容器 Caddy:8080 → axum:38424）")
    ),
    tags(
        (name = "devices", description = "设备注册与状态上报"),
        (name = "webrtc-signaling", description = "WebRTC 信令（offer/answer/SSE）"),
        (name = "health", description = "健康检查")
    ),
    paths(
        devices::register,
        devices::report_status,
        signaling_handlers::create_offer,
        signaling_handlers::poll_answer,
        signaling_handlers::subscribe_ticket,
        signaling_handlers::subscribe_events,
        signaling_handlers::submit_answer,
        crate::api::health,
    ),
    components(
        schemas(
            DeviceAddress,
            Platform,
            DeviceRegisterRequest,
            DeviceRegisterResponse,
            DeviceSummary,
            signaling_handlers::OfferRequest,
            signaling_handlers::OfferResponse,
            signaling_handlers::AnswerRequest,
            signaling_handlers::AnswerResponse,
            signaling_handlers::AnswerDecision,
            signaling_handlers::TicketResponse,
            StatusReportRequest,
            StatusReportResponse,
            crate::api::HealthResponse,
            ErrorBody,
            ErrorDetail,
        ),
        responses(
            BadRequest,
            Unauthorized,
            NotFound,
            RateLimited,
            DeviceOffline,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// 添加 securitySchemes（components 宏不支持，通过 Modify 注入）。
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("opaque")
                        .description(Some("注册时颁发的 64 字符 `pairing_token`。"))
                        .build(),
                ),
            );
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// 错误响应组件（与 specs/full/openapi/pairing-server.yaml#/components/responses 一致）
// ──────────────────────────────────────────────────────────────────────────

/// 请求参数无效（400 INVALID_REQUEST）。
#[allow(dead_code)]
#[derive(utoipa::ToResponse)]
#[response(
    description = "请求参数无效",
    example = json!({
        "error": {
            "code": "INVALID_REQUEST",
            "message": "The request payload is invalid",
            "details": { "field": "device_id" }
        }
    })
)]
pub struct BadRequest(ErrorBody);

/// Token 无效或已过期（401 TOKEN_INVALID）。
#[allow(dead_code)]
#[derive(utoipa::ToResponse)]
#[response(
    description = "Token 无效或已过期",
    example = json!({
        "error": {
            "code": "TOKEN_INVALID",
            "message": "The pairing token is invalid or expired"
        }
    })
)]
pub struct Unauthorized(ErrorBody);

/// 设备不存在（404 DEVICE_NOT_FOUND）。
#[allow(dead_code)]
#[derive(utoipa::ToResponse)]
#[response(
    description = "设备不存在",
    example = json!({
        "error": {
            "code": "DEVICE_NOT_FOUND",
            "message": "The device does not exist"
        }
    })
)]
pub struct NotFound(ErrorBody);

/// 速率超限（429 RATE_LIMITED）。
#[allow(dead_code)]
#[derive(utoipa::ToResponse)]
#[response(
    description = "速率超限（RATE_LIMIT 1 req/IP/秒）",
    example = json!({
        "error": {
            "code": "RATE_LIMITED",
            "message": "Too many requests"
        }
    })
)]
pub struct RateLimited(ErrorBody);

/// 桌面无活跃 SSE 订阅（503 DEVICE_OFFLINE，§7 fast-fail）。
#[allow(dead_code)]
#[derive(utoipa::ToResponse)]
#[response(
    description = "桌面无活跃 SSE 订阅（离线），offer 被快速拒绝",
    example = json!({
        "error": {
            "code": "DEVICE_OFFLINE",
            "message": "device not reachable / no active subscriber"
        }
    })
)]
pub struct DeviceOffline(ErrorBody);

// ──────────────────────────────────────────────────────────────────────────
// Swagger UI + 生成函数
// ──────────────────────────────────────────────────────────────────────────

/// Swagger UI 交互文档路由（仅 dev/test 挂载，见 `api::serve`）。
pub fn build_docs_router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/docs/swagger-ui").url("/docs/openapi.json", ApiDoc::openapi()))
}

/// 生成 (json, yaml) 文档内容。`gen-openapi` bin 与漂移测试共用此函数，
/// 保证导出文件与校验逻辑完全一致。
///
/// 两者统一以单个换行结尾（POSIX 文本约定 + prek `end-of-file-fixer` 要求），
/// 否则 prek 给产物补换行后会与 `generate_specs()` 的无换行输出不一致，
/// 导致 `tests/openapi.rs` 字节比对失败。
pub fn generate_specs() -> (String, String) {
    let spec = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&spec).expect("serialize openapi json");
    let yaml = spec.to_yaml().expect("serialize openapi yaml");
    (
        ensure_trailing_newline(&json),
        ensure_trailing_newline(&yaml),
    )
}

/// 确保字符串以恰好一个换行结尾。
fn ensure_trailing_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{s}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 文档包含全部 8 个路径。
    #[test]
    fn api_doc_contains_all_paths() {
        let spec = ApiDoc::openapi();
        assert_eq!(spec.paths.paths.len(), 8);
        for p in [
            "/api/devices",
            "/api/devices/{device_id}/status",
            "/api/devices/{device_id}/webrtc/offer",
            "/api/devices/{device_id}/webrtc/answer/{session_id}",
            "/api/devices/{device_id}/webrtc/subscribe",
            "/api/devices/{device_id}/webrtc/events",
            "/api/devices/{device_id}/webrtc/answer",
            "/health",
        ] {
            assert!(spec.paths.paths.contains_key(p), "missing path {p}");
        }
    }

    /// 文档包含 bearerAuth 安全方案。
    #[test]
    fn api_doc_contains_security_scheme() {
        let spec = ApiDoc::openapi();
        let components = spec.components.expect("components present");
        assert!(components.security_schemes.contains_key("bearerAuth"));
    }

    /// 文档为 OpenAPI 3.1.0。
    #[test]
    fn api_doc_is_openapi_31() {
        let spec = ApiDoc::openapi();
        assert_eq!(serde_json::to_value(&spec).unwrap()["openapi"], "3.1.0");
    }

    /// 生成函数输出确定性（连续两次调用一致，漂移检测前提）。
    #[test]
    fn generate_specs_is_deterministic() {
        let (json1, yaml1) = generate_specs();
        let (json2, yaml2) = generate_specs();
        assert_eq!(json1, json2);
        assert_eq!(yaml1, yaml2);
    }
}

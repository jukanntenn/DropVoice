//! 统一错误类型（spec 11 §5.1.3 错误码表）。
//!
//! `AppError` 通过 `IntoResponse` 转换为 OpenAPI 定义的统一错误响应格式：
//! `{ "error": { code, message, details? } }`。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// 机器可读错误码（`SCREAMING_SNAKE_CASE`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidRequest,
    TokenInvalid,
    DeviceNotFound,
    SessionNotFound,
    PayloadTooLarge,
    TooManySessions,
    RateLimited,
    /// 桌面无活跃 SSE 订阅（离线），手机 offer 被快速拒绝（§7 fast-fail）。
    DeviceOffline,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::TokenInvalid => "TOKEN_INVALID",
            ErrorCode::DeviceNotFound => "DEVICE_NOT_FOUND",
            ErrorCode::SessionNotFound => "SESSION_NOT_FOUND",
            ErrorCode::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
            ErrorCode::TooManySessions => "TOO_MANY_SESSIONS",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::DeviceOffline => "DEVICE_OFFLINE",
        }
    }
}

/// 内部错误类型，承载可映射到 HTTP 状态码的业务错误。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("token invalid or expired")]
    TokenInvalid,

    #[error("device not found")]
    DeviceNotFound,

    /// 信令会话不存在或已过期（TTL 60s 回收）。
    #[error("session not found or expired")]
    SessionNotFound,

    /// SDP 载荷超过上限（§4.6 64KB）。
    #[error("payload too large: {0} bytes (max {1})")]
    PayloadTooLarge(usize, usize),

    /// 每 device 并发会话超上限（§4.6 10）。
    #[error("too many concurrent sessions for device")]
    TooManySessions,

    #[error("rate limited")]
    RateLimited,

    /// 桌面无活跃 SSE 订阅（离线），手机 offer 被快速拒绝（§7 fast-fail，503）。
    #[error("device not reachable / no active subscriber")]
    DeviceOffline,

    /// 未映射到业务语义的内部错误（DB 故障等），对外统一 500。
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// sqlx 错误统一归为 Internal（500）。
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.into())
    }
}

impl AppError {
    /// 返回 (状态码, 错误码)。
    fn parts(&self) -> (StatusCode, ErrorCode) {
        match self {
            AppError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, ErrorCode::InvalidRequest),
            AppError::TokenInvalid => (StatusCode::UNAUTHORIZED, ErrorCode::TokenInvalid),
            AppError::DeviceNotFound => (StatusCode::NOT_FOUND, ErrorCode::DeviceNotFound),
            AppError::SessionNotFound => (StatusCode::NOT_FOUND, ErrorCode::SessionNotFound),
            AppError::PayloadTooLarge(_, _) => {
                (StatusCode::PAYLOAD_TOO_LARGE, ErrorCode::PayloadTooLarge)
            }
            AppError::TooManySessions => {
                (StatusCode::TOO_MANY_REQUESTS, ErrorCode::TooManySessions)
            }
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, ErrorCode::RateLimited),
            AppError::DeviceOffline => (StatusCode::SERVICE_UNAVAILABLE, ErrorCode::DeviceOffline),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InvalidRequest),
        }
    }
}

/// 错误响应体（与 `pairing-server.yaml#/components/schemas/Error` 一致）。
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    /// 机器可读错误码，`SCREAMING_SNAKE_CASE`。
    #[schema(value_type = String, example = "SESSION_NOT_FOUND")]
    pub code: &'static str,
    /// 人类可读错误信息。
    pub message: String,
    /// 可选调试详情。
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object, additional_properties = true)]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.parts();
        let message = if matches!(self, AppError::Internal(_)) {
            // 不泄露内部错误细节给客户端。
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        // 内部错误打 ERROR 日志，便于排障。
        if let AppError::Internal(e) = &self {
            tracing::error!(error = ?e, "pairing server internal error");
        }

        let body = ErrorBody {
            error: ErrorDetail {
                code: code.as_str(),
                message,
                details: None,
            },
        };
        (status, Json(body)).into_response()
    }
}

/// `Result` 别名，方便 handler 返回。
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    /// UT-error-01：各错误码 HTTP 状态正确。
    #[test]
    fn status_codes_are_correct() {
        assert_eq!(
            AppError::InvalidRequest("x".into()).parts().0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(AppError::TokenInvalid.parts().0, StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::DeviceNotFound.parts().0, StatusCode::NOT_FOUND);
        assert_eq!(AppError::SessionNotFound.parts().0, StatusCode::NOT_FOUND);
        assert_eq!(
            AppError::PayloadTooLarge(100_000, 65_536).parts().0,
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            AppError::TooManySessions.parts().0,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::RateLimited.parts().0,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::DeviceOffline.parts().0,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            AppError::Internal(anyhow::anyhow!("x")).parts().0,
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    /// UT-error-02：错误码字符串匹配 OpenAPI。
    #[test]
    fn error_codes_match_openapi() {
        assert_eq!(ErrorCode::InvalidRequest.as_str(), "INVALID_REQUEST");
        assert_eq!(ErrorCode::TokenInvalid.as_str(), "TOKEN_INVALID");
        assert_eq!(ErrorCode::DeviceNotFound.as_str(), "DEVICE_NOT_FOUND");
        assert_eq!(ErrorCode::SessionNotFound.as_str(), "SESSION_NOT_FOUND");
        assert_eq!(ErrorCode::PayloadTooLarge.as_str(), "PAYLOAD_TOO_LARGE");
        assert_eq!(ErrorCode::TooManySessions.as_str(), "TOO_MANY_SESSIONS");
        assert_eq!(ErrorCode::RateLimited.as_str(), "RATE_LIMITED");
        assert_eq!(ErrorCode::DeviceOffline.as_str(), "DEVICE_OFFLINE");
    }

    /// UT-error-03：IntoResponse 体形状。
    #[tokio::test]
    async fn error_response_body_shape() {
        let err = AppError::SessionNotFound;
        let resp = err.into_response();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("error").is_some());
        assert_eq!(json["error"]["code"], "SESSION_NOT_FOUND");
        assert!(json["error"]["message"].as_str().is_some());
        // 无 details 时无 details 键。
        assert!(json["error"].get("details").is_none());
    }

    /// UT-error-04：Internal 隐藏 message。
    #[tokio::test]
    async fn internal_error_hides_details() {
        let err = AppError::Internal(anyhow::anyhow!("secret database password"));
        let resp = err.into_response();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["message"], "internal server error");
        assert!(!json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("secret"));
    }

    /// UT-error-05：From<sqlx::Error>。
    #[test]
    fn from_sqlx_error() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err: AppError = sqlx_err.into();
        assert!(matches!(app_err, AppError::Internal(_)));
    }
}

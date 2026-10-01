//! HTTP client for the pairing/signaling server (webrtc-scan-direct-design §4.1).
//!
//! Provides two operations:
//! - `register_device` — `POST /api/devices`
//! - `report_status` — `PUT /api/devices/{id}/status`
//!
//! 配对码（code）改由桌面本地生成 + 本地验证（§3.2），服务端零 code 知识，
//! 故不再有 `generate_pairing_code` 调用。
//!
//! All calls are best-effort with structured error returns; the caller decides
//! retry and degradation strategy.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 解析信令服务器 base URL —— 桌面端唯一入口（heartbeat 与 webview 的
/// `get_signaling_url` 命令共用）。
///
/// 优先级：env `PAIRING_SERVER_URL`（开发编排专用，.vscode/tasks.json 内置，
/// 打包应用不设）→ 配置文件 `network.pairing_server_url`（默认
/// [`crate::config::DEFAULT_PAIRING_SERVER_URL`]，PWA 与 API 同源，§10.4）。
pub fn resolve_base_url(config_url: &str) -> String {
    std::env::var("PAIRING_SERVER_URL").unwrap_or_else(|_| config_url.to_string())
}

/// Timeout for individual HTTP requests (spec 11 §4 `SIGNALING_TIMEOUT` 3s).
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// SSE 订阅的 chunk 级活性超时：服务器每 15s 具名 ping，连续 3 个周期无任何
/// 字节即视为半开（隧道掐断 / 系统休眠唤醒后的死 socket）→ 上层重建订阅。
const SSE_READ_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(45);

/// Build the shared HTTP client.
///
/// `PAIRING_SERVER_INSECURE=1` skips TLS certificate verification — reserved
/// for self-signed TLS deployments. Never set in production.
fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    build_client_with_timeout(Some(REQUEST_TIMEOUT))
}

/// SSE 专用客户端：无总超时（长连接），以 chunk 级 read_timeout 保证活性。
fn build_streaming_client() -> Result<reqwest::Client, reqwest::Error> {
    build_client_with_timeout(None)
}

/// 按超时策略构建客户端（None = 不设总超时，供流式响应）。
fn build_client_with_timeout(
    timeout: Option<std::time::Duration>,
) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder();
    builder = match timeout {
        Some(t) => builder.timeout(t),
        None => builder.read_timeout(SSE_READ_IDLE_TIMEOUT),
    };
    let insecure = std::env::var("PAIRING_SERVER_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build()
}

/// Request body for `POST /devices` (spec 11 §5.1.1).
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub device_id: Uuid,
    pub platform: String,
    pub address: RegisterAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAddress {
    pub ip: String,
    pub port: u16,
}

/// Response body for `POST /devices`.
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub pairing_token: String,
    #[serde(default)]
    pub device_name: Option<String>,
}

/// Request body for `PUT /devices/{id}/status` (spec 11 §7.4).
#[derive(Debug, Serialize)]
pub struct StatusReportRequest {
    pub address: RegisterAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
}

/// Response body for `PUT /devices/{id}/status`.
#[derive(Debug, Deserialize)]
pub struct StatusReportResponse {
    pub ok: bool,
}

/// Error from the pairing server API.
#[derive(Debug, thiserror::Error)]
pub enum PairingClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Server returned error: {status} {code}")]
    Server {
        status: u16,
        code: String,
        message: String,
    },
}

/// Register this device with the pairing server.
///
/// Returns the pairing token on success (201 Created or 200 OK).
/// `existing_token`：本地持久化的当前 token（如有）——已存在设备的复用/轮换
/// 都要求携带它（凭据模型：token 只发给持有者，裸重注册会被 401）。
/// 401 表示 device_id 已被其他身份占用（或服务端数据丢失）——调用方应重置
/// 设备身份后重试（heartbeat.rs）。
pub async fn register_device(
    base_url: &str,
    device_id: Uuid,
    ip: &str,
    port: u16,
    device_name: Option<&str>,
    existing_token: Option<&str>,
) -> Result<RegisterResponse, PairingClientError> {
    let client = build_client()?;

    let req = RegisterRequest {
        device_id,
        platform: "desktop".into(),
        address: RegisterAddress {
            ip: ip.to_string(),
            port,
        },
        device_name: device_name.map(|s| s.to_string()),
    };

    let url = format!("{base_url}/api/devices");
    let mut request = client.post(&url).json(&req);
    if let Some(token) = existing_token {
        request = request.bearer_auth(token);
    }
    let resp = request.send().await?;
    interpret_register_response(resp).await
}

/// 解读注册响应（新建 201 / 复用轮换 200 / 凭据缺失 401）。
async fn interpret_register_response(
    resp: reqwest::Response,
) -> Result<RegisterResponse, PairingClientError> {
    if resp.status().is_success() {
        let body: RegisterResponse = resp.json().await?;
        Ok(body)
    } else {
        Err(error_from_response(resp).await)
    }
}

/// 非成功状态 → 结构化 Server 错误（body 统一错误格式 `{error:{code,message}}`）。
async fn error_from_response(resp: reqwest::Response) -> PairingClientError {
    let status = resp.status().as_u16();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    PairingClientError::Server {
        status,
        code: body["error"]["code"]
            .as_str()
            .unwrap_or("UNKNOWN")
            .to_string(),
        message: body["error"]["message"].as_str().unwrap_or("").to_string(),
    }
}

/// Report device status to the pairing server (spec 11 §7.4).
pub async fn report_status(
    base_url: &str,
    device_id: Uuid,
    token: &str,
    ip: &str,
    port: u16,
    device_name: Option<&str>,
) -> Result<StatusReportResponse, PairingClientError> {
    let client = build_client()?;

    let req = StatusReportRequest {
        address: RegisterAddress {
            ip: ip.to_string(),
            port,
        },
        device_name: device_name.map(|s| s.to_string()),
    };

    let url = format!("{base_url}/api/devices/{device_id}/status");
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&req)
        .send()
        .await?;

    if resp.status().is_success() {
        let body: StatusReportResponse = resp.json().await?;
        Ok(body)
    } else {
        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(PairingClientError::Server {
            status,
            code: body["error"]["code"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            message: body["error"]["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// SSE 推送的 incoming offer 载荷（§4.1 端点 3）。
#[derive(Debug, Clone, Deserialize)]
pub struct SseOfferEvent {
    pub session_id: String,
    pub credential: String,
    pub sdp: String,
}

/// 桌面对 offer 的应答决策（accepted 携带 answer SDP；rejected 携带原因）。
///
/// serde tag 序列化为 `{"status":"accepted","sdp":...}` / `{"status":"rejected",
/// "reason":...}`——与服务器 `AnswerRequest` 的 wire 格式一致（§4.1 端点 4）。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum AnswerDecision {
    Accepted { sdp: String },
    Rejected { reason: String },
}

/// 换取 SSE 一次性订阅票据（POST /api/devices/{id}/webrtc/subscribe，Bearer）。
///
/// 长效 pairing_token 不进 URL（Caddy 访问日志记录完整 uri 含 query）；
/// 票据 60s 单次使用，出现在日志中无害。认证失败返回 401 Server 错误。
pub async fn fetch_ticket(
    base_url: &str,
    device_id: Uuid,
    token: &str,
) -> Result<String, PairingClientError> {
    let client = build_client()?;

    #[derive(serde::Deserialize)]
    struct TicketResponse {
        ticket: String,
    }

    let url = format!("{base_url}/api/devices/{device_id}/webrtc/subscribe");
    let resp = client.post(&url).bearer_auth(token).send().await?;

    if resp.status().is_success() {
        let body: TicketResponse = resp.json().await?;
        Ok(body.ticket)
    } else {
        Err(error_from_response(resp).await)
    }
}

/// 订阅 SSE 事件流（GET /api/devices/{id}/webrtc/events?ticket=，§4.1 端点 3）。
///
/// `ticket` 由 [`fetch_ticket`] 以 Bearer 换取（60s 单次使用）。
/// 返回响应字节流，由调用方用 `eventsource_stream::EventStream` 解析。
/// 客户端已设 chunk 级 read_timeout（45s）——服务器 15s ping 一次，连续
/// 3 个周期无字节即报错，半开连接（隧道静默掐断、休眠唤醒死 socket）由
/// 此确定性检出，无需任何客户端侧生命周期定时器。
///
/// 票据无效（过期/已用）返回 `PairingClientError::Server { status: 401 }`。
pub async fn subscribe_events(
    base_url: &str,
    device_id: Uuid,
    ticket: &str,
) -> Result<impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>>, PairingClientError> {
    let client = build_streaming_client()?;

    let url = format!("{base_url}/api/devices/{device_id}/webrtc/events?ticket={ticket}");
    let resp = client
        .get(&url)
        .header("Accept", "text/event-stream")
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(resp.bytes_stream())
    } else {
        Err(error_from_response(resp).await)
    }
}

/// 回填 answer（POST /api/devices/{id}/webrtc/answer，Bearer，§4.1 端点 4）。
pub async fn submit_answer(
    base_url: &str,
    device_id: Uuid,
    token: &str,
    session_id: &str,
    decision: &AnswerDecision,
) -> Result<(), PairingClientError> {
    let client = build_client()?;

    let url = format!("{base_url}/api/devices/{device_id}/webrtc/answer");
    let body = serde_json::json!({
        "session_id": session_id,
        "sdp": match decision {
            AnswerDecision::Accepted { sdp } => Some(sdp),
            AnswerDecision::Rejected { .. } => None,
        },
        "status": match decision {
            AnswerDecision::Accepted { .. } => "accepted",
            AnswerDecision::Rejected { .. } => "rejected",
        },
        "reason": match decision {
            AnswerDecision::Accepted { .. } => None,
            AnswerDecision::Rejected { reason } => Some(reason),
        },
    });
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(PairingClientError::Server {
            status,
            code: body["error"]["code"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            message: body["error"]["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// env 是进程全局且测试并发，无法安全 unset；两种外部状态下的结果都合法。
    #[test]
    fn resolve_base_url_env_wins_over_config_value() {
        let resolved = resolve_base_url("http://configured.example");
        match std::env::var("PAIRING_SERVER_URL") {
            Ok(v) => assert_eq!(resolved, v),
            Err(_) => assert_eq!(resolved, "http://configured.example"),
        }
    }

    #[test]
    fn register_request_serializes_correctly() {
        let req = RegisterRequest {
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            platform: "desktop".into(),
            address: RegisterAddress {
                ip: "192.168.1.100".into(),
                port: 38425,
            },
            device_name: Some("My PC".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["platform"], "desktop");
        assert_eq!(json["address"]["ip"], "192.168.1.100");
        assert_eq!(json["address"]["port"], 38425);
        assert_eq!(json["device_name"], "My PC");
    }

    #[test]
    fn register_request_omits_device_name_when_none() {
        let req = RegisterRequest {
            device_id: Uuid::new_v4(),
            platform: "desktop".into(),
            address: RegisterAddress {
                ip: "10.0.0.1".into(),
                port: 38425,
            },
            device_name: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("device_name").is_none());
    }

    #[test]
    fn status_report_request_serializes() {
        let req = StatusReportRequest {
            address: RegisterAddress {
                ip: "10.0.0.1".into(),
                port: 38425,
            },
            device_name: Some("New Name".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["address"]["ip"], "10.0.0.1");
        assert_eq!(json["device_name"], "New Name");
    }

    /// answer 回填 wire 格式：accepted 携带 sdp，rejected 携带 reason（§4.1 端点 4）。
    #[test]
    fn answer_decision_serializes_to_wire_format() {
        let accepted = AnswerDecision::Accepted { sdp: "v=0".into() };
        let json = serde_json::to_value(&accepted).unwrap();
        assert_eq!(json["status"], "accepted");
        assert_eq!(json["sdp"], "v=0");

        let rejected = AnswerDecision::Rejected {
            reason: "invalid_credential".into(),
        };
        let json = serde_json::to_value(&rejected).unwrap();
        assert_eq!(json["status"], "rejected");
        assert_eq!(json["reason"], "invalid_credential");
    }

    /// SSE offer 事件载荷反序列化（字段名与服务器 §4.1 端点 3 一致）。
    #[test]
    fn sse_offer_event_deserializes() {
        let ev: SseOfferEvent =
            serde_json::from_str(r#"{"session_id":"s1","credential":"123456","sdp":"v=0"}"#)
                .unwrap();
        assert_eq!(ev.session_id, "s1");
        assert_eq!(ev.credential, "123456");
        assert_eq!(ev.sdp, "v=0");
    }
}

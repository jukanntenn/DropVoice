//! WebRTC 信令端点（webrtc-scan-direct-design §4.1）。
//!
//! 五个端点：
//! - `POST /api/devices/{id}/webrtc/offer` —— 手机发起
//! - `GET  /api/devices/{id}/webrtc/answer/{session_id}` —— 手机长轮询（hold 30s）
//! - `POST /api/devices/{id}/webrtc/subscribe` —— 桌面换取 SSE 一次性票据（Bearer）
//! - `GET  /api/devices/{id}/webrtc/events` —— 桌面 SSE 长连接（query ticket）
//! - `POST /api/devices/{id}/webrtc/answer` —— 桌面回填（Bearer）
//!
//! 速率豁免（§4.8）：SSE + answer 长轮询 + 票据/回填建在独立 Router
//! （build_exempt_router），merge 到 app，不继承 route_layer(rate_limit)。
//! POST offer 保持限速。豁免端点各自有并发帽兜底（config::MAX_CONCURRENT_*）。

use std::convert::Infallible;
use std::sync::atomic::Ordering;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api::auth::AuthenticatedDevice;
use crate::api::signaling::{AnswerStatus, OfferEvent};
use crate::api::state::AppState;
use crate::config::{
    LONG_POLL_HOLD, MAX_CONCURRENT_POLLS, MAX_CONCURRENT_SSE, SDP_MAX_BYTES, SSE_MAX_LIFETIME,
    SSE_PING_INTERVAL,
};
use crate::docs::{BadRequest, DeviceOffline, NotFound, RateLimited, Unauthorized};
use crate::error::{AppError, AppResult};
use crate::store::device_repo;

// ──────────────────────────────────────────────────────────────────────────
// 请求/响应 schema
// ──────────────────────────────────────────────────────────────────────────

/// `POST /webrtc/offer` 请求体。code（首次配对）或 token（重连）二选一。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OfferRequest {
    /// 6 位配对码（首次配对）。
    #[serde(default)]
    pub code: Option<String>,
    /// `dvct_` 连接令牌（重连）。
    #[serde(default)]
    pub token: Option<String>,
    /// 手机生成的 offer SDP。
    pub sdp: String,
}

/// `POST /webrtc/offer` 响应体。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OfferResponse {
    pub session_id: String,
    pub device: device_repo::DeviceSummary,
}

/// `POST /webrtc/answer` 请求体（桌面回填）。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AnswerRequest {
    pub session_id: String,
    /// accepted 时必填，rejected 时省略。
    #[serde(default)]
    pub sdp: Option<String>,
    /// `accepted` 或 `rejected`。
    pub status: AnswerDecision,
    /// rejected 时的原因（如 `invalid_credential`）。
    #[serde(default)]
    pub reason: Option<String>,
}

/// answer 决策。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnswerDecision {
    Accepted,
    Rejected,
}

/// `GET /webrtc/answer/{session_id}` 响应体。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AnswerResponse {
    /// accepted 时有值。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp: Option<String>,
    pub status: AnswerDecision,
    /// rejected 时的原因。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────
// 端点 1：POST /api/devices/{id}/webrtc/offer（手机发起）
// ──────────────────────────────────────────────────────────────────────────

/// 手机发起配对/重连：校验 deviceId 存在 → 存 offer → SSE 推送桌面 → 返回 session_id。
#[utoipa::path(
    post,
    path = "/api/devices/{device_id}/webrtc/offer",
    tag = "webrtc-signaling",
    operation_id = "createOffer",
    summary = "手机发起 WebRTC offer（配对/重连）",
    description = "手机扫码后发起。body 带 code（首次配对）或 token（重连）+ sdp。\n\
        服务器不校验 code/token（§3.2），只验证 device 存在，建 session（60s TTL），\n\
        通过 SSE 推送给桌面。credential 字段透传给桌面校验。\n\
        每 device 并发会话上限 10（§4.6），超限 429。SDP 上限 64KB。",
    params(
        ("device_id" = Uuid, Path, format = "uuid", description = "目标桌面设备 ID")
    ),
    request_body(
        content = OfferRequest,
        content_type = "application/json",
        examples(
            ("first_pair" = (summary = "首次配对", value = json!({
                "code": "123456", "sdp": "v=0\r\n..."
            }))),
            ("reconnect" = (summary = "重连", value = json!({
                "token": "dvct_xxx", "sdp": "v=0\r\n..."
            })))
        )
    ),
    responses(
        (status = 200, description = "session 创建成功", body = OfferResponse,
            example = json!({
                "session_id": "9f1c-...",
                "device": { "id": "550e8400-...", "name": "My PC" }
            })),
        (status = 400, response = BadRequest),
        (status = 404, response = NotFound),
        (status = 413, description = "SDP 载荷超过 64KB"),
        (status = 429, response = RateLimited),
        (status = 503, response = DeviceOffline)
    )
)]
#[tracing::instrument(
    name = "webrtc.offer",
    skip_all,
    fields(device_id = %device_id)
)]
pub async fn create_offer(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<OfferRequest>,
) -> AppResult<Json<OfferResponse>> {
    // credential 转换规则（§3.2）：credential = code.or(token)，两者缺失 400。
    let credential = req
        .code
        .clone()
        .or_else(|| req.token.clone())
        .ok_or_else(|| AppError::InvalidRequest("code or token required".into()))?;

    // SDP 载荷上限（§4.6 64KB）。
    if req.sdp.len() > SDP_MAX_BYTES {
        return Err(AppError::PayloadTooLarge(req.sdp.len(), SDP_MAX_BYTES));
    }

    // 校验 device 存在 + 取摘要（§4.1 响应 {device:{name}}）。
    let summary = device_repo::find_device_summary(&state.pool, device_id).await?;

    // 建会话（§4.6 TTL 60s，每 device 上限 10）。
    let session_id = state
        .signaling
        .create_session(device_id, req.sdp.clone(), credential.clone())
        .await
        .map_err(|_| AppError::TooManySessions)?;

    state
        .metrics
        .webrtc_offers_received_total
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // SSE 推送给桌面。无活跃订阅（桌面离线/未就绪）→ 503 fast-fail（§7），
    // 让手机立即进入退避而非建会话干等 30s answer 超时。
    let session = state
        .signaling
        .get_session(&session_id)
        .await
        .ok_or(AppError::SessionNotFound)?;
    let pushed = state.signaling.push_offer(&session).await;
    if !pushed {
        tracing::warn!(%session_id, %device_id, "no active SSE subscriber; rejecting offer");
        return Err(AppError::DeviceOffline);
    }
    tracing::info!(%session_id, %device_id, pushed, "offer created");

    Ok(Json(OfferResponse {
        session_id,
        device: summary,
    }))
}

// ──────────────────────────────────────────────────────────────────────────
// 端点 2：GET /api/devices/{id}/webrtc/answer/{session_id}（手机长轮询）
// ──────────────────────────────────────────────────────────────────────────

/// 手机长轮询：hold 最多 30s，answer 就绪立即返回，无则 204 超时。
/// session_id 本身即为凭据（高熵不可猜），无需额外认证（§4.1 认证策略差异说明）。
/// 并发等待受全局上限保护（豁免限流端点的资源封顶）。
#[utoipa::path(
    get,
    path = "/api/devices/{device_id}/webrtc/answer/{session_id}",
    tag = "webrtc-signaling",
    operation_id = "pollAnswer",
    summary = "手机长轮询 answer（hold 30s）",
    description = "手机 POST offer 后长轮询此端点。\n\
        - answer 就绪 → 200 立即返回（accepted/rejected）\n\
        - 30s 内无 answer → 204（桌面离线或 SSE 未就绪）\n\
        - session 过期/不存在 → 404\n\
        session_id 是手机刚 POST offer 拿到的（高熵），无需额外认证。\n\
        全局并发等待有上限，超限返回 429。",
    params(
        ("device_id" = Uuid, Path, format = "uuid", description = "目标桌面设备 ID"),
        ("session_id" = String, Path, description = "POST offer 返回的会话 ID")
    ),
    responses(
        (status = 200, description = "answer 就绪", body = AnswerResponse),
        (status = 204, description = "answer 尚未就绪（30s 超时），继续轮询"),
        (status = 404, response = NotFound),
        (status = 429, response = RateLimited)
    )
)]
#[tracing::instrument(
    name = "webrtc.poll_answer",
    skip_all,
    fields(session_id = %session_id)
)]
pub async fn poll_answer(
    State(state): State<AppState>,
    Path((_device_id, session_id)): Path<(Uuid, String)>,
) -> AppResult<Response> {
    // 全局并发等待帽（守卫 Drop 保证释放；上限外直接 429，不占资源）。
    if state.poll_active.load(Ordering::Relaxed) >= MAX_CONCURRENT_POLLS {
        return Err(AppError::RateLimited);
    }
    let _poll_guard = ConnGuard::new(&state.poll_active);

    // 双检模式（§4.6）：先查共享状态，再 await Notify。
    let (answer, notify) = state
        .signaling
        .peek_answer(&session_id)
        .await
        .map_err(|_| AppError::SessionNotFound)?;

    if let Some(ans) = answer {
        return Ok(Json(answer_response(ans)).into_response());
    }

    // answer 未就绪，await Notify（最多 hold 30s）。
    let notify = notify.ok_or(AppError::SessionNotFound)?;

    // 等待 answer 或超时。
    let hold_result = tokio::time::timeout(LONG_POLL_HOLD, notify.notified()).await;

    // 超时后再查一次（避免 notify 已发但 answer 未写入的极小窗口——双检）。
    let (answer, _) = state
        .signaling
        .peek_answer(&session_id)
        .await
        .map_err(|_| AppError::SessionNotFound)?;
    match answer {
        Some(ans) => Ok(Json(answer_response(ans)).into_response()),
        None => {
            // 超时无 answer → 204（桌面离线或 SSE 未就绪）。
            let _ = hold_result; // 显式忽略超时结果
            Ok(StatusCode::NO_CONTENT.into_response())
        }
    }
}

/// 并发计数守卫：构造 +1，Drop -1（早退/异常路径同样释放）。
struct ConnGuard<'a> {
    gauge: &'a std::sync::atomic::AtomicUsize,
}

impl<'a> ConnGuard<'a> {
    fn new(gauge: &'a std::sync::atomic::AtomicUsize) -> Self {
        gauge.fetch_add(1, Ordering::Relaxed);
        Self { gauge }
    }
}

impl Drop for ConnGuard<'_> {
    fn drop(&mut self) {
        self.gauge.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 将 AnswerStatus 转为响应 JSON。
fn answer_response(ans: AnswerStatus) -> AnswerResponse {
    match ans {
        AnswerStatus::Accepted { sdp } => AnswerResponse {
            sdp: Some(sdp),
            status: AnswerDecision::Accepted,
            reason: None,
        },
        AnswerStatus::Rejected { reason } => AnswerResponse {
            sdp: None,
            status: AnswerDecision::Rejected,
            reason: Some(reason),
        },
    }
}

// ──────────────────────────────────────────────────────────────────────────
// 端点 3：POST /api/devices/{id}/webrtc/subscribe（桌面换取 SSE 票据）
// ──────────────────────────────────────────────────────────────────────────

/// `POST /webrtc/subscribe` 响应体。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TicketResponse {
    /// 一次性 SSE 订阅票据（60s TTL，单次使用）。
    pub ticket: String,
}

/// 桌面换取 SSE 订阅票据（Bearer）。SSE 用 `?ticket=` 建连。
///
/// 长效 pairing_token 不进 URL：Caddy 访问日志记录完整 uri（含 query），
/// token 出现在日志即等于泄露。票据单次使用、60s 过期，出现在日志中无害。
#[utoipa::path(
    post,
    path = "/api/devices/{device_id}/webrtc/subscribe",
    tag = "webrtc-signaling",
    operation_id = "subscribeTicket",
    summary = "桌面换取 SSE 一次性订阅票据",
    description = "桌面持 Bearer pairing_token 调用，换取 60 秒单次使用的订阅票据，\n\
        随后以 `GET .../webrtc/events?ticket=` 建立 SSE 长连接。\n\
        票据与 device_id 绑定，兑换即失效。",
    params(
        ("device_id" = Uuid, Path, format = "uuid", description = "桌面自身设备 ID")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "票据签发", body = TicketResponse),
        (status = 401, response = Unauthorized)
    )
)]
#[tracing::instrument(name = "webrtc.subscribe", skip_all, fields(device_id = %device_id))]
pub async fn subscribe_ticket(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    auth: AuthenticatedDevice,
) -> AppResult<Json<TicketResponse>> {
    // Bearer 对应设备必须与路径一致。
    if auth.0 != device_id {
        return Err(AppError::TokenInvalid);
    }
    let ticket = state.signaling.issue_ticket(device_id).await;
    Ok(Json(TicketResponse { ticket }))
}

// ──────────────────────────────────────────────────────────────────────────
// 端点 4：GET /api/devices/{id}/webrtc/events（桌面 SSE 长连接）
// ──────────────────────────────────────────────────────────────────────────

/// events 端点的 query 参数（一次性票据）。
#[derive(Debug, Default, Deserialize)]
pub struct EventsQuery {
    /// `POST .../webrtc/subscribe` 签发的一次性票据。
    ticket: Option<String>,
}

/// 桌面 SSE 长连接：推送 incoming offer，每 15s 心跳。
/// 认证：query `ticket`（一次性，Bearer 换取；长效 token 不进 URL/日志）。
#[utoipa::path(
    get,
    path = "/api/devices/{device_id}/webrtc/events",
    tag = "webrtc-signaling",
    operation_id = "subscribeEvents",
    summary = "桌面 SSE 长连接（一次性票据认证）",
    description = "桌面维持 1 条 SSE 长连接，接收手机的 incoming offer。\n\
        认证用 query 参数 `?ticket=`（由 `POST .../webrtc/subscribe` 以 Bearer 换取，\n\
        60s 单次使用——长效 pairing_token 不进 URL，避免进入访问日志）。\n\
        每 15s 发具名 `ping` 事件（防 CF 125s 超时 + 连接活性监测）。\n\
        offer 事件 data 格式：`{session_id, credential, sdp}`。",
    params(
        ("device_id" = Uuid, Path, format = "uuid", description = "桌面自身设备 ID"),
        ("ticket" = String, Query, description = "一次性订阅票据（Bearer 换取）")
    ),
    responses(
        (status = 200, description = "SSE 事件流", content_type = "text/event-stream",
            example = "event: offer\ndata: {\"session_id\":\"...\",\"credential\":\"123456\",\"sdp\":\"v=0...\"}\n\nevent: ping\ndata: {\"ts\":1723000000}\n\n"),
        (status = 401, response = Unauthorized),
        (status = 429, response = RateLimited)
    )
)]
#[tracing::instrument(
    name = "webrtc.events",
    skip_all,
    fields(device_id = %device_id)
)]
pub async fn subscribe_events(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Query(query): Query<EventsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 一次性票据兑换：单次使用、限期、设备绑定。
    let ticket = query
        .ticket
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or(AppError::TokenInvalid)?;
    if !state.signaling.redeem_ticket(ticket, device_id).await {
        return Err(AppError::TokenInvalid);
    }

    // 全局并发 SSE 帽：超限拒绝（每桌面本就只允许 1 条，上限即防滥用）。
    if state.sse_active.load(Ordering::Relaxed) >= MAX_CONCURRENT_SSE {
        return Err(AppError::RateLimited);
    }
    let sse_gauge = state.sse_active.clone();
    sse_gauge.fetch_add(1, Ordering::Relaxed);

    // 订阅该 device 的 offer 推送。
    let sub = state.signaling.subscribe_offers(device_id).await;
    let store = state.signaling.clone();

    // 构建事件流：spawn 一个任务用 tokio::select! 合并 offer 推送 + 15s ping，
    // 写入 mpsc 通道；Sse 消费 ReceiverStream。
    // 当 SSE 连接断开，axum drop 响应 future → ReceiverStream drop → rx drop →
    // 任务 select 分支都返回 None/关闭 → 任务退出（释放并发计数）。
    let (event_tx, event_rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    tokio::spawn(async move {
        let mut rx = sub.rx;
        // 该订阅的唯一身份；退出时只 compare-and-delete 自己，绝不误删替换自己的新订阅（§7）。
        let sub_id = sub.sub_id;
        let mut ping = tokio::time::interval(SSE_PING_INTERVAL);
        ping.tick().await; // 消费立即触发的第一个 tick（interval 特性）
                           // 生命周期上限：中间层（内网穿透/CF）有连接时长上限（实测 openresty 300s），
                           // 超限被掐断时客户端可能不重连（实测）。240s 时优雅关闭 + retry 提示，
                           // EventSource 按 retry 间隔自动重连（订阅由 subscribe_offers insert 覆盖）。
        let mut lifetime = tokio::time::interval(SSE_MAX_LIFETIME);
        lifetime.tick().await; // 首个 tick 立即触发，忽略
        loop {
            tokio::select! {
                biased; // 优先处理 offer
                maybe_event = rx.recv() => {
                    match maybe_event {
                        Some(event) => {
                            let evt = offer_sse_event(&event);
                            if event_tx.send(Ok(evt)).await.is_err() {
                                // 接收端关闭（SSE 断开），退出。
                                break;
                            }
                        }
                        None => break, // 订阅被替换/移除
                    }
                }
                _ = ping.tick() => {
                    let ts = chrono::Utc::now().timestamp();
                    let evt = Event::default()
                        .event("ping")
                        .data(serde_json::json!({"ts": ts}).to_string());
                    if event_tx.send(Ok(evt)).await.is_err() {
                        break;
                    }
                }
                _ = lifetime.tick() => {
                    // 到达生命周期：发 retry 提示（客户端 ~1s 后重连），优雅关闭。
                    let retry_evt = Event::default().retry(Duration::from_millis(1000));
                    if event_tx.send(Ok(retry_evt)).await.is_err() {
                        break;
                    }
                    break;
                }
            }
        }
        // 退订：SSE 连接结束后移除该 device 的订阅。
        // §7：带 sub_id 做 compare-and-delete——若本订阅已被新订阅替换
        //（sub_id 不匹配），则不删除新订阅，避免老任务误杀新连接。
        store.unsubscribe_offers(device_id, sub_id).await;
        // 释放全局 SSE 并发计数。
        sse_gauge.fetch_sub(1, Ordering::Relaxed);
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(event_rx);
    Ok((
        // §10.4：SSE 必须流式。axum 只发 content-type + cache-control 两个 header，
        // 不自动发 X-Accel-Buffering: no；nginx/openresty 前置时（如内网穿透）默认
        // 缓冲整个响应，SSE 永不完成 → 客户端收不到任何事件。此 header 让 nginx
        // 对该响应关闭缓冲。
        [("x-accel-buffering", "no")],
        Sse::new(stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        ),
    ))
}

/// 构造 offer SSE 事件。
fn offer_sse_event(event: &OfferEvent) -> Event {
    Event::default().event("offer").data(
        serde_json::json!({
            "session_id": event.session_id,
            "credential": event.credential,
            "sdp": event.sdp,
        })
        .to_string(),
    )
}

// ──────────────────────────────────────────────────────────────────────────
// 端点 4：POST /api/devices/{id}/webrtc/answer（桌面回填，Bearer）
// ──────────────────────────────────────────────────────────────────────────

/// 桌面回填 answer → 唤醒手机的长轮询。
#[utoipa::path(
    post,
    path = "/api/devices/{device_id}/webrtc/answer",
    tag = "webrtc-signaling",
    operation_id = "submitAnswer",
    summary = "桌面回填 answer",
    description = "桌面收到 SSE offer 后，校验 credential（本地内存比对 code/token），\n\
        生成 answer SDP 回填。accepted 时附 sdp，rejected 时附 reason（如 invalid_credential）。\n\
        回填后唤醒手机的长轮询。",
    params(
        ("device_id" = Uuid, Path, format = "uuid", description = "桌面自身设备 ID")
    ),
    security(("bearerAuth" = [])),
    request_body(
        content = AnswerRequest,
        content_type = "application/json",
        examples(
            ("accepted" = (summary = "接受", value = json!({
                "session_id": "9f1c...", "sdp": "v=0\r\n...", "status": "accepted"
            }))),
            ("rejected" = (summary = "拒绝（credential 错误）", value = json!({
                "session_id": "9f1c...", "status": "rejected", "reason": "invalid_credential"
            })))
        )
    ),
    responses(
        (status = 200, description = "回填成功"),
        (status = 401, response = Unauthorized),
        (status = 404, response = NotFound)
    )
)]
#[tracing::instrument(
    name = "webrtc.submit_answer",
    skip_all,
    fields(device_id = %device_id, session_id = %req.session_id)
)]
pub async fn submit_answer(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    auth: AuthenticatedDevice,
    Json(req): Json<AnswerRequest>,
) -> AppResult<StatusCode> {
    // 路径 device_id 必须与认证 token 对应的设备一致。
    if auth.0 != device_id {
        return Err(AppError::TokenInvalid);
    }

    let answer = match req.status {
        AnswerDecision::Accepted => {
            let sdp = req
                .sdp
                .ok_or_else(|| AppError::InvalidRequest("sdp required for accepted".into()))?;
            AnswerStatus::Accepted { sdp }
        }
        AnswerDecision::Rejected => AnswerStatus::Rejected {
            reason: req.reason.unwrap_or_else(|| "rejected".into()),
        },
    };

    state
        .signaling
        .submit_answer(&req.session_id, answer)
        .await
        .map_err(|_| AppError::SessionNotFound)?;

    state
        .metrics
        .webrtc_answers_completed_total
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    // 端点逻辑主要通过 SignalStore 单测 + e2e 测试覆盖（见 tests/e2e/）。
    // 此处的 schema 序列化用例验证 wire 格式。

    use super::*;

    #[test]
    fn offer_response_serializes_correctly() {
        let resp = OfferResponse {
            session_id: "abc".into(),
            device: device_repo::DeviceSummary {
                id: Uuid::nil(),
                name: Some("My PC".into()),
            },
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["session_id"], "abc");
        assert_eq!(json["device"]["name"], "My PC");
    }

    #[test]
    fn answer_request_deserializes_accepted() {
        let json = r#"{"session_id":"s1","sdp":"v=0","status":"accepted"}"#;
        let req: AnswerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, AnswerDecision::Accepted);
        assert_eq!(req.sdp.as_deref(), Some("v=0"));
    }

    #[test]
    fn answer_request_deserializes_rejected() {
        let json = r#"{"session_id":"s1","status":"rejected","reason":"invalid_credential"}"#;
        let req: AnswerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, AnswerDecision::Rejected);
        assert_eq!(req.reason.as_deref(), Some("invalid_credential"));
    }

    #[test]
    fn answer_response_accepted_has_sdp() {
        let resp = AnswerResponse {
            sdp: Some("v=0".into()),
            status: AnswerDecision::Accepted,
            reason: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["sdp"], "v=0");
        assert_eq!(json["status"], "accepted");
        assert!(json.get("reason").is_none());
    }

    #[test]
    fn answer_response_rejected_has_reason() {
        let resp = AnswerResponse {
            sdp: None,
            status: AnswerDecision::Rejected,
            reason: Some("invalid_credential".into()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("sdp").is_none());
        assert_eq!(json["status"], "rejected");
        assert_eq!(json["reason"], "invalid_credential");
    }

    #[test]
    fn offer_sse_event_format() {
        let event = OfferEvent {
            session_id: "s1".into(),
            credential: "123456".into(),
            sdp: "v=0".into(),
        };
        // offer_sse_event 构造带 event:offer + data 的 Event（axum SSE Event item）。
        let _evt = offer_sse_event(&event);
        // SSE Event 渲染验证在 e2e 测试（需真实 SSE 端到端）。
    }
}

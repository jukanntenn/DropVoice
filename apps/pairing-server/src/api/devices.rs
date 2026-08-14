//! `POST /devices` 与 `PUT /devices/{id}/status` 处理器（spec 11 §5.1.1）。

use std::sync::atomic::Ordering;
use std::time::Instant;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::api::auth::AuthenticatedDevice;
use crate::api::error::generate_token;
use crate::api::state::AppState;
use crate::docs::{BadRequest, NotFound, RateLimited, Unauthorized};
use crate::domain::{
    DeviceRegisterRequest, DeviceRegisterResponse, StatusReportRequest, StatusReportResponse,
};
use crate::error::AppResult;
use crate::store::device_repo::{self, RegisterOutcome};

/// `POST /devices` —— 幂等 upsert（spec 11 §5.1.1）。
#[utoipa::path(
    post,
    path = "/api/devices",
    tag = "devices",
    operation_id = "registerDevice",
    summary = "设备注册（幂等 upsert）",
    description = "桌面端本地生成持久化 UUID v4 作为 device_id（首次启动后永不变更）。\n本端点为幂等 upsert 语义：\n- device_id 不存在 → 创建记录，颁发 token → 201 Created\n- device_id 存在且 token 有效 → 返回现有记录（含原 token）→ 200 OK\n- device_id 存在但 token 已过期 → 刷新 token，返回新 token → 200 OK\n重复注册是预期内的正常行为（应用重启、网络重连、IP 变化都会触发），不是冲突，不返回 409。",
    request_body(
        content = DeviceRegisterRequest,
        content_type = "application/json",
        examples(("new_device" = (summary = "首次注册", value = json!({
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "platform": "desktop",
            "device_name": "My PC",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))))
    ),
    responses(
        (status = 201, description = "新建设备成功，颁发 token", body = DeviceRegisterResponse,
            example = json!({
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "platform": "desktop",
                "device_name": "My PC",
                "address": { "ip": "192.168.1.100", "port": 38425 },
                "pairing_token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
                "created_at": "2026-07-18T12:00:00Z"
            })),
        (status = 200, description = "设备已存在（复用或续期）。返回现有记录；若 token 已过期，返回新 token",
            body = DeviceRegisterResponse),
        (status = 400, response = BadRequest),
        (status = 429, response = RateLimited)
    )
)]
#[tracing::instrument(
    name = "device.register",
    skip_all,
    fields(device_id = %req.device_id, platform = ?req.platform)
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<DeviceRegisterRequest>,
) -> AppResult<(StatusCode, Json<DeviceRegisterResponse>)> {
    let started = Instant::now();

    // Pre-req-10: validate device_name length.
    req.validate()?;

    let token = generate_token();

    let outcome = device_repo::upsert(&state.pool, &req, &token, state.clock.as_ref()).await?;

    let (device, status) = match outcome {
        RegisterOutcome::Created(d) => {
            state.cache.put_token(&token, req.device_id).await;
            state
                .metrics
                .devices_registered_total
                .fetch_add(1, Ordering::Relaxed);
            (d, StatusCode::CREATED)
        }
        RegisterOutcome::Reused { device, old_token } => {
            // Pre-req-3：续期时失效旧 token 缓存。
            if let Some(ref old) = old_token {
                state.cache.invalidate_token(old).await;
            }
            state
                .cache
                .put_token(&device.pairing_token, device.id)
                .await;
            (device, StatusCode::OK)
        }
    };

    let is_new = status == StatusCode::CREATED;
    tracing::info!(
        is_new,
        duration_ms = started.elapsed().as_millis() as u64,
        "device registered"
    );

    Ok((status, Json(device.into())))
}

/// `PUT /devices/{id}/status` —— 状态/地址上报（spec 11 §7.4）。
#[utoipa::path(
    put,
    path = "/api/devices/{device_id}/status",
    tag = "devices",
    operation_id = "reportStatus",
    summary = "状态/地址上报（心跳）",
    description = "桌面端周期性上报（DEVICE_STATUS_INTERVAL 5 分钟），IP 变化时立即触发额外上报\n（节流 IP_REPORT_THROTTLE 10 秒）。用于：① 续期 token；② 标记设备在线；\n③ IP 变化时刷新服务端记录。\n返回 401 表示 token 过期，客户端应自动重 POST /devices（同 device_id）触发续期，\n再重放本次心跳（§7.5）。",
    params(
        ("device_id" = Uuid, Path, format = "uuid",
            description = "客户端生成的 UUID v4 设备标识",
            example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    security(("bearerAuth" = [])),
    request_body(
        content = StatusReportRequest,
        content_type = "application/json",
        examples(
            ("heartbeat" = (summary = "周期心跳", value = json!({
                "address": { "ip": "192.168.1.100", "port": 38425 }
            }))),
            ("ip_changed" = (summary = "IP 变化上报", value = json!({
                "address": { "ip": "192.168.1.105", "port": 38425 },
                "device_name": "Renamed PC"
            })))
        )
    ),
    responses(
        (status = 200, description = "上报成功", body = StatusReportResponse),
        (status = 401, response = Unauthorized),
        (status = 404, response = NotFound)
    )
)]
#[tracing::instrument(
    name = "status.report",
    skip_all,
    fields(device_id = %device_id, ip_changed = tracing::field::Empty)
)]
pub async fn report_status(
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
    auth: AuthenticatedDevice,
    Json(req): Json<StatusReportRequest>,
) -> AppResult<Json<StatusReportResponse>> {
    // 路径 device_id 必须与认证 token 对应的设备一致。
    if auth.0 != device_id {
        return Err(crate::error::AppError::TokenInvalid);
    }

    // Pre-req-10: validate device_name length.
    req.validate()?;

    // 入队批量写入（spec 11 §10.2，心跳 best-effort）。
    // Pre-req-7：传递 device_name。
    state
        .batch
        .enqueue(
            device_id,
            &req.address,
            req.device_name.clone(),
            state.clock.as_ref(),
        )
        .await;

    tracing::Span::current().record("ip_changed", tracing::field::display(&req.address.ip));

    Ok(Json(StatusReportResponse { ok: true }))
}

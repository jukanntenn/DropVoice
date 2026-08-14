use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::commands::{AppState, ClientEvent, ClientInfo, ConnectionInfo, PairingCodePayload};
use crate::config::{DeviceConfig, DropVoiceConfig};
use crate::connection::{ConnectionState, CredentialKind, PairingCode};
use crate::error::{AppError, AppResult};
use crate::network::heartbeat;
use crate::text::validate_text;

/// 启动连接服务（webrtc-scan-direct-design §5）。
///
/// 新架构下不再绑定 TCP/axum 服务器——WebRTC 信令客户端运行在 webview JS，
/// Rust 侧只做：① 签发配对码；② 启动 heartbeat（注册 + emit pairing_token）；
/// ③ ConnectionManager 就绪（注入队列）。
#[tauri::command]
pub async fn start_server(app: AppHandle, state: State<'_, AppState>) -> AppResult<ConnectionInfo> {
    // 拒绝重复启动。
    {
        let guard = state.connection_state.lock().await;
        if guard.is_some() {
            return Err(AppError::ServerAlreadyRunning);
        }
    }

    // 构建 ConnectionState。
    let config_snapshot = state.config.read().await.clone();
    let cs = ConnectionState::new(config_snapshot);
    cs.mark_started().await;

    // 启动注入队列处理任务。
    let delay_ms = state.config.read().await.injection.delay_ms;
    let cm = cs.connection_manager.clone();
    let metrics = cs.metrics.clone();
    cm.start_processing_task(delay_ms, metrics);

    // 签发配对码（§3.2，6 位数字）。
    let pairing_code = cs.issue_pairing_code().await;

    // 启动 heartbeat（注册 + emit pairing_token，§5.1 新增链路）。
    let config = state.config.clone();
    let get_ip: heartbeat::GetIpFn = Arc::new(get_local_ip);
    let heartbeat_handle = heartbeat::start_heartbeat(app.clone(), config, get_ip);
    {
        let mut guard = state.heartbeat.lock().await;
        *guard = Some(heartbeat_handle);
    }

    // 构建 QR 载荷（§6.5；纯读当前码，cs 此时仍 owned）。
    let qr_payload = {
        let device = cs.config.read().await.device.clone();
        build_qr_payload(&cs.pairing_code, &device).await
    };

    // 持久化 ConnectionState。
    {
        let mut guard = state.connection_state.lock().await;
        *guard = Some(cs);
    }

    // §6 启动配对码轮换定时器：自主到点轮换并 push `pairing_code_rotated` 事件，
    // 把"何时轮换"（后端时间职责）与"如何展示"（前端职责）解耦。get_connection_info
    // 不再触发轮换（消除 double-call + 轮询固有延迟）。
    let rotation_handle = spawn_pairing_code_rotation(
        app.clone(),
        state.connection_state.clone(),
        state.config.clone(),
    );
    {
        let mut guard = state.rotation_task.lock().await;
        *guard = Some(rotation_handle);
    }

    info!(pairing_code = %pairing_code, "connection service started");
    Ok(ConnectionInfo {
        running: true,
        qr_payload: Some(qr_payload),
        active_connections: 0,
        pairing_code: Some(pairing_code),
        clients: vec![],
        queue_depth: 0,
    })
}

/// 停止连接服务 + heartbeat + 配对码轮换定时器。
#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> AppResult<()> {
    // 停止 heartbeat。
    let heartbeat_handle = {
        let mut guard = state.heartbeat.lock().await;
        guard.take()
    };
    if let Some(handle) = heartbeat_handle {
        handle.stop().await;
        info!("heartbeat stopped");
    } else {
        warn!("stop_server called but no service is running");
    }

    // §6 停止配对码轮换定时器。
    let rotation_handle = {
        let mut guard = state.rotation_task.lock().await;
        guard.take()
    };
    if let Some(handle) = rotation_handle {
        handle.abort();
        info!("pairing code rotation task aborted");
    }

    // 清除 ConnectionState。
    {
        let mut guard = state.connection_state.lock().await;
        if let Some(cs) = guard.take() {
            cs.mark_stopped().await;
        }
    }

    Ok(())
}

/// 返回当前连接信息（前端轮询/查询）。
#[tauri::command]
pub async fn get_connection_info(state: State<'_, AppState>) -> AppResult<ConnectionInfo> {
    // §6：get_connection_info 不再触发配对码轮换（轮换由独立定时器负责）。
    // 数据提取在锁内完成并克隆所需的 Arc（pairing_code/config），释放锁后再
    // 调 build_qr_payload——它读 pairing_code 锁，与 connection_state 锁不同，
    // 避免任何重入风险。
    let (active, pairing_code, clients, queue_depth, pairing_code_lock, config_lock) = {
        let guard = state.connection_state.lock().await;
        match guard.as_ref() {
            Some(cs) => {
                let active = cs.connection_manager.count().await;
                // 纯读当前码（不轮换，§6）。
                let pairing_code = cs.current_pairing_code().await;
                let clients: Vec<ClientInfo> = cs
                    .connection_manager
                    .get_all()
                    .await
                    .into_iter()
                    .map(|c| ClientInfo {
                        id: c.id,
                        connected_at: c.connected_at.to_rfc3339(),
                    })
                    .collect();
                let queue_depth = cs.connection_manager.queue_depth();
                (
                    active,
                    pairing_code,
                    clients,
                    queue_depth,
                    cs.pairing_code.clone(),
                    cs.config.clone(),
                )
            }
            None => {
                return Ok(ConnectionInfo {
                    running: false,
                    qr_payload: None,
                    active_connections: 0,
                    pairing_code: None,
                    clients: vec![],
                    queue_depth: 0,
                });
            }
        }
    };

    // build_qr_payload 纯读当前码（不轮换）。
    let device = config_lock.read().await.device.clone();
    let qr_payload = build_qr_payload(&pairing_code_lock, &device).await;

    Ok(ConnectionInfo {
        running: true,
        qr_payload: Some(qr_payload),
        active_connections: active,
        pairing_code,
        clients,
        queue_depth,
    })
}

/// 注入文本（webview JS 收到 DataChannel {type:text} 后调用，§5.1/§5.4）。
#[tauri::command]
pub async fn inject_text(
    text: String,
    client_id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    validate_text(&text)?;
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    cs.connection_manager
        .enqueue_injection(client_id.clone(), text.clone())
        .map_err(|e| {
            warn!(client_id = %client_id, error = %e, "inject_text rejected");
            e
        })?;
    info!(client_id = %client_id, chars = text.chars().count(), "text injection enqueued");
    Ok(())
}

/// 持久化连接令牌（webview JS 收到桌面签发后调用，§5.1）。
///
/// 注：实际签发由桌面侧完成（首次配对成功 + DataChannel open 后，
/// JS 调用 issue_connection_token），此命令用于手机持久化的场景。
/// 当前架构下签发在桌面侧，此命令保留供未来扩展。
#[tauri::command]
pub async fn save_token(token: String, state: State<'_, AppState>) -> AppResult<()> {
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    cs.add_connection_token(token).await;
    Ok(())
}

/// 签发新连接令牌并返回（首次配对成功 + DataChannel open 后 JS 调用，§5.1/§6.3）。
///
/// 桌面侧签发 `dvct_` 令牌，持久化（FIFO 上限 100），返回给 JS 通过
/// DataChannel 发给手机 `{type:"token",token}`。
#[tauri::command]
pub async fn issue_connection_token(state: State<'_, AppState>) -> AppResult<String> {
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    Ok(cs.issue_connection_token().await)
}

/// 校验 credential（webview JS 收到 SSE offer 后调用，§5.1/§5.7）。
///
/// 桌面本地比对 code（5min TTL）或 token（内存）。返回命中的 credential 类型，
/// JS 据此决定生成 accepted 还是 rejected answer。
#[tauri::command]
pub async fn validate_credential(
    credential: String,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    let kind = cs.validate_credential(&credential).await;
    let result = match kind {
        CredentialKind::Code => "code",
        CredentialKind::Token => "token",
        CredentialKind::Invalid => "invalid",
    };
    Ok(result.to_string())
}

/// 同步刷新 pairing_token（webview JS SSE 收 401 时调用，决策点③，§5.7）。
///
/// 触发一次注册流程获取新 token，持久化 + emit pairing_token。
/// JS 收到 emit 后重建 SSE 连接。
#[tauri::command]
pub async fn refresh_pairing_token(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let config = state.config.clone();
    let get_ip: heartbeat::GetIpFn = Arc::new(get_local_ip);
    heartbeat::refresh_pairing_token(app, config, get_ip)
        .await
        .map_err(|e| AppError::ServerStartFailed { reason: e })
}

/// 注册一个已连接的手机（DataChannel open 后 JS 调用）。
#[tauri::command]
pub async fn register_client(
    client_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    cs.connection_manager.register(client_id.clone()).await?;
    info!(client_id = %client_id, "client registered (DataChannel open)");
    drop(guard);
    // §0.3/§6：push `client_registered`，前端据此关闭"添加设备"浮层（§1）。
    if let Err(e) = app.emit("client_registered", ClientEvent { client_id }) {
        warn!(error = %e, "failed to emit client_registered event");
    }
    Ok(())
}

/// 注销一个已连接的手机（DataChannel close 后 JS 调用）。
#[tauri::command]
pub async fn unregister_client(client_id: String, state: State<'_, AppState>) -> AppResult<()> {
    let guard = state.connection_state.lock().await;
    let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
        reason: "connection service not running".into(),
    })?;
    cs.connection_manager.unregister(&client_id).await;
    info!(client_id = %client_id, "client unregistered (DataChannel closed)");
    Ok(())
}

/// 返回当前持久化的 pairing_token（§5.1 启动引导兜底）。
///
/// heartbeat 注册成功即 emit "pairing_token"，但 webview JS 的 listen() 注册
/// 晚于 emit 时事件丢失（Tauri 事件不排队、不重发）。JS 侧在事件未及时到达时
/// 轮询本命令获取 token（heartbeat 已持久化到 config）。
#[tauri::command]
pub async fn get_pairing_token(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let cfg = state.config.read().await;
    Ok(cfg.device.pairing_token.clone())
}

/// 构建 QR 载荷（§6.5：dropvoice://pair?code=&device=&name=）。
///
/// §6：纯读当前配对码（不轮换）。命令路径（get_connection_info）与定时器路径
/// （rotation timer）共用，避免轮换逻辑重复。
pub async fn build_qr_payload(
    pairing_code: &Arc<RwLock<PairingCode>>,
    device: &DeviceConfig,
) -> String {
    let code = pairing_code
        .read()
        .await
        .as_ref()
        .map(|(c, _)| c.clone())
        .unwrap_or_default();
    let device_id = device.device_id.clone();
    let device_name = device.device_name.clone();
    // percent-encode name（保留字母数字和常见安全字符，其余编码）。
    let name_encoded: String = device_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf).as_bytes();
                bytes.iter().map(|b| format!("%{b:02X}")).collect()
            }
        })
        .collect();
    format!("dropvoice://pair?code={code}&device={device_id}&name={name_encoded}")
}

/// §6 配对码轮换定时器：周期 = pairing_code_expiry_minutes。到点自主轮换，
/// 若发生轮换则 push `pairing_code_rotated` 事件（前端即时刷新 QR/链接/重连码）。
/// stop_server 通过 abort 取消。
fn spawn_pairing_code_rotation(
    app: AppHandle,
    connection_state: Arc<Mutex<Option<ConnectionState>>>,
    config: Arc<RwLock<DropVoiceConfig>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let interval_minutes = config
            .read()
            .await
            .security
            .pairing_code_expiry_minutes
            .max(1);
        let duration = std::time::Duration::from_secs(interval_minutes * 60);
        info!(
            rotation_secs = duration.as_secs(),
            "pairing code rotation timer started"
        );
        loop {
            tokio::time::sleep(duration).await;
            let guard = connection_state.lock().await;
            let Some(cs) = guard.as_ref() else {
                // 服务已停止；退出定时器。
                break;
            };
            // 检测是否真的轮换（compare before/after）。old 是 Option<String>，
            // new 是 String；轮换发生当且仅当前后不同。
            let old = cs.current_pairing_code().await;
            let new = cs.rotate_pairing_code_if_expired().await;
            if old.as_deref() != Some(new.as_str()) {
                let device = cs.config.read().await.device.clone();
                let qr_payload = build_qr_payload(&cs.pairing_code, &device).await;
                info!(pairing_code = %new, "pairing code rotated by timer; emitting event");
                if let Err(e) = app.emit(
                    "pairing_code_rotated",
                    PairingCodePayload {
                        code: new.clone(),
                        qr_payload,
                    },
                ) {
                    warn!(error = %e, "failed to emit pairing_code_rotated event");
                }
            }
        }
        info!("pairing code rotation timer exited");
    })
}

/// 取本机 IP（简化版，用于 heartbeat 注册）。
fn get_local_ip() -> Option<String> {
    // 使用 local_ip_address crate（已在依赖中）。
    local_ip_address::local_ip().ok().map(|ip| ip.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn get_local_ip_returns_something() {
        // 不强断言具体值（CI 环境可能无 LAN IP），只验证不 panic。
        let _ = get_local_ip();
    }

    /// §6：build_qr_payload 纯读当前码 + percent-encode 名称；不轮换。
    #[tokio::test]
    async fn build_qr_payload_reads_code_and_encodes_name() {
        let pairing_code: Arc<RwLock<PairingCode>> =
            Arc::new(RwLock::new(Some(("123456".to_string(), Utc::now()))));
        let device = DeviceConfig {
            device_id: "dev-1".into(),
            device_name: "我的 PC".into(),
            pairing_token: None,
            connected_tokens: None,
        };
        let payload = build_qr_payload(&pairing_code, &device).await;
        assert!(payload.starts_with("dropvoice://pair?code=123456&device=dev-1&name="));
        // 中文 percent-encoded（非 ASCII 安全字符被编码）。
        assert!(!payload.contains("我的"));
        assert!(payload.contains("%"));
    }

    /// §6：无配对码时 build_qr_payload 返回空 code 段（不 panic）。
    #[tokio::test]
    async fn build_qr_payload_empty_when_no_code() {
        let pairing_code: Arc<RwLock<PairingCode>> = Arc::new(RwLock::new(None));
        let device = DeviceConfig {
            device_id: "dev-1".into(),
            device_name: "PC".into(),
            pairing_token: None,
            connected_tokens: None,
        };
        let payload = build_qr_payload(&pairing_code, &device).await;
        assert_eq!(payload, "dropvoice://pair?code=&device=dev-1&name=PC");
    }
}

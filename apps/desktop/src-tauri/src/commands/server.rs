use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use tokio::sync::{watch, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::commands::{
    AppState, ClientInfo, ConnectionInfo, InjectionResultPayload, PairingCodePayload,
};
use crate::config::{DeviceConfig, DropVoiceConfig};
use crate::connection::{ConnectionState, InjectionResultCallback, PairingCode};
use crate::error::{AppError, AppResult};
use crate::network::heartbeat;
use crate::network::signaling;

/// 启动连接服务（webrtc-scan-direct-design §5）。
///
/// 不绑定 TCP 端口——信令面与 WebRTC 应答面全部为 Rust 常驻子系统：
/// ① 签发配对码；② heartbeat（注册 + pairing_token 经 watch 分发）；
/// ③ signaling 监督任务（SSE 订阅 + offer 应答 + DataChannel 接线）；
/// ④ ConnectionManager 就绪（注入队列）。webview 只做视图。
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

    // 启动注入队列处理任务。on_result 回调把注入结果推送到前端事件总线
    // （injection_completed / injection_failed），供 webview console 观测与
    // 未来 UI 提示；ConnectionManager 本身保持与 Tauri 解耦。
    let delay_ms = state.config.read().await.injection.delay_ms;
    let cm = cs.connection_manager.clone();
    let metrics = cs.metrics.clone();
    let app_for_events = app.clone();
    let on_result: Arc<InjectionResultCallback> =
        Arc::new(move |client_id, success, chars, elapsed_ms| {
            let payload = InjectionResultPayload {
                client_id: client_id.to_string(),
                success,
                chars,
                elapsed_ms,
            };
            let event = if success {
                "injection_completed"
            } else {
                "injection_failed"
            };
            if let Err(e) = app_for_events.emit(event, payload) {
                warn!(error = %e, "failed to emit {event} event");
            }
        });
    cm.start_processing_task(delay_ms, metrics, Some(on_result));

    // 签发配对码（§3.2，6 位数字）。
    let pairing_code = cs.issue_pairing_code().await;

    // pairing_token 总线：heartbeat 注册成功 → 分发给 signaling 监督任务。
    let get_ip: heartbeat::GetIpFn = Arc::new(get_local_ip);
    let (token_tx, token_rx) = watch::channel(None);

    // 启动 heartbeat（注册 + 心跳 + token 分发）。
    let heartbeat_handle =
        heartbeat::start_heartbeat(state.config.clone(), get_ip.clone(), token_tx.clone());
    {
        let mut guard = state.heartbeat.lock().await;
        *guard = Some(heartbeat_handle);
    }

    // 启动信令监督任务（SSE 订阅 + WebRTC 应答面）。
    let signaling_handle = signaling::start_signaling(
        app.clone(),
        state.config.clone(),
        state.connection_state.clone(),
        token_rx,
        token_tx,
        get_ip,
    );
    {
        let mut guard = state.signaling.lock().await;
        *guard = Some(signaling_handle);
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

/// 停止连接服务：信令监督（SSE + 应答会话）→ heartbeat → 轮换定时器 → 状态。
#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> AppResult<()> {
    // 先停信令监督任务：其收尾会注销全部客户端（依赖尚未清除的
    // ConnectionState），并关闭全部 RTCPeerConnection。
    let signaling_handle = {
        let mut guard = state.signaling.lock().await;
        guard.take()
    };
    if let Some(handle) = signaling_handle {
        handle.stop().await;
        info!("signaling supervisor stopped");
    }

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

/// 本机 IP（简化版，用于 heartbeat 注册）。
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

//! Heartbeat / 设备注册任务（webrtc-scan-direct-design §5.1 职责边界）。
//!
//! Rust 侧负责设备注册（POST /api/devices）+ 周期心跳（PUT /api/devices/{id}/status），
//! 维持 pairing_token。注册成功后通过 `watch` channel 把 token 分发给同进程的
//! 信令监督任务（`network::signaling`，SSE 订阅据此建连/重建）——token 全程
//! 不出 Rust 进程，webview 不再是信令链路的一环。
//!
//! 配对码改由桌面启动时本地生成（§3.2），不再涉及服务器 code-gen 端点。
//!
//! 时间常量：
//! - `DEVICE_STATUS_INTERVAL` = 5 分钟（心跳周期）
//! - `HEARTBEAT_BACKOFF_MAX` = 5 分钟
//! - `REGISTER_BACKOFF_MAX` = 16 秒

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{watch, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::DropVoiceConfig;
use crate::network::pairing_client;

/// pairing_token 进程内分发总线（注册成功 → Some(token)；SSE 401 重注册亦走此）。
pub type TokenBus = watch::Sender<Option<String>>;

/// 心跳周期（5 分钟）。
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// 心跳失败后最大退避（5 分钟）。
const BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);

/// 连续失败阈值（超过后只记日志，本地连接不受影响）。
const STATUS_FAIL_THRESHOLD: u32 = 5;

/// 注册退避最大值（16 秒）。
const REGISTER_BACKOFF_MAX: Duration = Duration::from_secs(16);

/// Heartbeat 句柄，stop 时优雅关闭。
pub struct HeartbeatHandle {
    stop_tx: watch::Sender<bool>,
    join: tokio::task::JoinHandle<()>,
}

impl HeartbeatHandle {
    pub async fn stop(self) {
        let _ = self.stop_tx.send(true);
        let _ = self.join.await;
    }
}

/// 取本机 IP 的回调（注入便于测试）。
pub type GetIpFn = Arc<dyn Fn() -> Option<String> + Send + Sync>;

/// 启动注册 + 心跳后台任务。
///
/// 1. 向配对服务器注册（指数退避重试直到成功或关闭）
/// 2. 持久化 pairing_token 到 config
/// 3. 经 `token_bus` 通知信令监督任务（SSE 订阅重建）
/// 4. 周期心跳（PUT status），token 过期（401）→ 重新注册 → 刷新 token → 分发
pub fn start_heartbeat(
    config: Arc<RwLock<DropVoiceConfig>>,
    get_ip: GetIpFn,
    token_bus: TokenBus,
) -> HeartbeatHandle {
    let (stop_tx, mut stop_rx) = watch::channel(false);

    let join = tokio::spawn(async move {
        info!("heartbeat task starting (registration + periodic status report)");

        let token = register_with_backoff(&config, &get_ip, &token_bus, &mut stop_rx).await;

        let token = match token {
            Some(t) => t,
            None => {
                info!("heartbeat task stopping (registration abandoned)");
                return;
            }
        };

        heartbeat_loop(&config, &token, &get_ip, &token_bus, &mut stop_rx).await;
    });

    HeartbeatHandle { stop_tx, join }
}

/// 向配对服务器注册，指数退避重试直到成功或关闭信号。
async fn register_with_backoff(
    config: &Arc<RwLock<DropVoiceConfig>>,
    get_ip: &GetIpFn,
    token_bus: &TokenBus,
    stop_rx: &mut watch::Receiver<bool>,
) -> Option<String> {
    let mut backoff = Duration::from_secs(1);
    let mut identity_reset_done = false;

    loop {
        let (base_url, device_id, device_name, port, saved_token) = {
            let cfg = config.read().await;
            (
                pairing_client::resolve_base_url(&cfg.network.pairing_server_url),
                cfg.device.device_id.clone(),
                cfg.device.device_name.clone(),
                cfg.server.port,
                cfg.device.pairing_token.clone(),
            )
        };

        let device_id = match Uuid::parse_str(&device_id) {
            Ok(id) => id,
            Err(e) => {
                error!(error = %e, "invalid device_id in config");
                return None;
            }
        };

        let ip = get_ip().unwrap_or_else(|| "127.0.0.1".into());

        match pairing_client::register_device(
            &base_url,
            device_id,
            &ip,
            port,
            Some(&device_name),
            saved_token.as_deref(),
        )
        .await
        {
            Ok(resp) => {
                info!(
                    token_len = resp.pairing_token.len(),
                    "registered with pairing server"
                );
                let token = resp.pairing_token.clone();
                // 持久化 token 到 config。
                {
                    let mut cfg = config.write().await;
                    cfg.device.pairing_token = Some(token.clone());
                    if let Err(e) = cfg.save() {
                        warn!(error = %e, "failed to persist pairing token");
                    }
                }
                // 分发给信令监督任务（SSE 据此建连/重建）。
                token_bus.send_replace(Some(token.clone()));
                return Some(token);
            }
            Err(pairing_client::PairingClientError::Server { status: 401, .. })
                if !identity_reset_done =>
            {
                // 凭据被拒：device_id 已被其他身份占用（本地 token 丢失或服务端
                // 数据重置）。重置身份（新 UUID → 注册必走新建分支），一次为限。
                // 副作用：已配对的手机需重新扫码。
                warn!("register rejected (401); resetting device identity (rescan required)");
                identity_reset_done = true;
                {
                    let mut cfg = config.write().await;
                    cfg.device.device_id = Uuid::new_v4().to_string();
                    cfg.device.pairing_token = None;
                    if let Err(e) = cfg.save() {
                        error!(error = %e, "failed to persist identity reset");
                        return None;
                    }
                }
                continue; // 立即用新身份重试，不消耗退避。
            }
            Err(e) => {
                warn!(error = %e, backoff = ?backoff, "registration failed, retrying");
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = stop_rx.changed() => {
                        info!("registration abandoned (shutdown)");
                        return None;
                    }
                }
                backoff = (backoff * 2).min(REGISTER_BACKOFF_MAX);
            }
        }
    }
}

/// 主心跳循环：周期 PUT status，401 时重新注册刷新 token 并分发。
async fn heartbeat_loop(
    config: &Arc<RwLock<DropVoiceConfig>>,
    initial_token: &str,
    get_ip: &GetIpFn,
    token_bus: &TokenBus,
    stop_rx: &mut watch::Receiver<bool>,
) {
    let mut token = initial_token.to_string();
    let mut consecutive_failures: u32 = 0;
    let mut backoff = Duration::from_secs(1);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(HEARTBEAT_INTERVAL) => {}
            _ = stop_rx.changed() => {
                info!("heartbeat task stopping");
                break;
            }
        }

        let (base_url, device_id, device_name, port) = {
            let cfg = config.read().await;
            (
                pairing_client::resolve_base_url(&cfg.network.pairing_server_url),
                cfg.device.device_id.clone(),
                cfg.device.device_name.clone(),
                cfg.server.port,
            )
        };

        let device_id = match Uuid::parse_str(&device_id) {
            Ok(id) => id,
            Err(e) => {
                error!(error = %e, "invalid device_id, skipping heartbeat");
                continue;
            }
        };

        let ip = get_ip().unwrap_or_else(|| "127.0.0.1".into());

        match pairing_client::report_status(
            &base_url,
            device_id,
            &token,
            &ip,
            port,
            Some(&device_name),
        )
        .await
        {
            Ok(_) => {
                consecutive_failures = 0;
                backoff = Duration::from_secs(1);
                debug!("heartbeat OK");
            }
            Err(pairing_client::PairingClientError::Server { status: 401, .. }) => {
                // Token 过期 → 重新注册 → 刷新 token → 分发。
                warn!("token expired (401), re-registering");
                match register_with_backoff(config, get_ip, token_bus, stop_rx).await {
                    Some(new_token) => {
                        token = new_token;
                        consecutive_failures = 0;
                    }
                    None => break, // Shutdown.
                }
            }
            Err(e) => {
                consecutive_failures += 1;
                warn!(
                    error = %e,
                    failures = consecutive_failures,
                    "heartbeat failed"
                );

                if consecutive_failures >= STATUS_FAIL_THRESHOLD {
                    warn!("status report failures exceeded threshold, local connection unaffected");
                }

                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = stop_rx.changed() => break,
                }
                backoff = (backoff * 2).min(BACKOFF_MAX);
            }
        }
    }
}

/// 同步刷新 pairing_token（信令监督任务 SSE 收 401 时调用，决策点③）。
///
/// 触发一次注册流程获取新 token，持久化 + 经 `token_bus` 分发。返回新 token
/// 供调用方使用。与 heartbeat_loop 的 401 分支共享 register 逻辑。
pub async fn refresh_pairing_token(
    config: Arc<RwLock<DropVoiceConfig>>,
    get_ip: GetIpFn,
    token_bus: TokenBus,
) -> Result<String, String> {
    // 用一个永远不触发的 stop_rx（刷新不应被中断）。
    let (_stop_tx, mut stop_rx) = watch::channel(false);
    register_with_backoff(&config, &get_ip, &token_bus, &mut stop_rx)
        .await
        .ok_or_else(|| "failed to refresh pairing token".to_string())
}

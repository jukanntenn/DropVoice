//! 信令监督任务 —— SSE 订阅生命周期 + offer 分发（替代 webview 版 SignalingClient）。
//!
//! 与 heartbeat 同进程同时钟：pairing_token 经 `watch` 总线分发（注册成功 /
//! 401 重注册均发布），token 变更即重建 SSE。重连策略只有一条确定性机制：
//! **chunk 级读超时**（`pairing_client::subscribe_events` 的 reqwest
//! read_timeout 45s）——半开隧道、系统休眠唤醒后的死 socket、服务器 240s
//! 生命周期关闭，全部表现为流结束/读错误，由此统一触发重建。tokio 定时器
//! 在系统唤醒后立即触发过期的睡眠（已在日志中验证），无需任何 wake 钩子。
//!
//! 收到 offer → `webrtc::spawn_offer_session`（独立任务，不阻塞 SSE 循环）。

use std::sync::Arc;
use std::time::Duration;

use eventsource_stream::EventStream;
use futures_util::StreamExt;
use tauri::AppHandle;
use tokio::sync::{watch, Mutex, RwLock};
use tracing::{info, warn};

use crate::config::DropVoiceConfig;
use crate::connection::ConnectionState;
use crate::network::heartbeat::{self, GetIpFn, TokenBus};
use crate::network::pairing_client::{self, SseOfferEvent};
use crate::webrtc::{self, AnswerContext, AnswerEndpoint};

/// 网络错误后的重连退避上限（与 heartbeat REGISTER_BACKOFF_MAX 对齐）。
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(16);

/// 服务器 240s 生命周期优雅关闭携带的 `retry:` 提示为 ~1s。
const SERVER_RETRY_HINT: Duration = Duration::from_secs(1);

/// 信令监督任务句柄；stop 时优雅关闭（SSE 断开 + 全部应答会话清理）。
pub struct SignalingHandle {
    stop_tx: watch::Sender<bool>,
    join: tokio::task::JoinHandle<()>,
}

impl SignalingHandle {
    pub async fn stop(self) {
        let _ = self.stop_tx.send(true);
        let _ = self.join.await;
    }
}

/// 启动信令监督任务。
///
/// `token_rx` 消费 heartbeat 发布的 pairing_token（首个 token 到达前阻塞
/// 等待）；SSE 401 时经 `token_bus` + `get_ip` 主动触发重注册。
pub fn start_signaling(
    app: AppHandle,
    config: Arc<RwLock<DropVoiceConfig>>,
    connection_state: Arc<Mutex<Option<ConnectionState>>>,
    token_rx: watch::Receiver<Option<String>>,
    token_bus: TokenBus,
    get_ip: GetIpFn,
) -> SignalingHandle {
    let (stop_tx, stop_rx) = watch::channel(false);

    let join = tokio::spawn(async move {
        info!("signaling supervisor starting");
        run_supervisor(
            Arc::new(AnswerContext::new(app, connection_state)),
            config,
            token_rx,
            token_bus,
            get_ip,
            stop_rx,
        )
        .await;
        info!("signaling supervisor stopped");
    });

    SignalingHandle { stop_tx, join }
}

/// SSE 单次运行的可能退出原因（决定重连策略）。
#[derive(Debug)]
enum StreamExit {
    /// stop_server。
    Stopped,
    /// 服务器优雅关闭（240s 生命周期 `retry:` 提示）或流自然结束。
    Ended,
    /// heartbeat 发布了新 token（401 重注册 / 轮换）。
    TokenChanged,
    /// SSE 认证失败（401）→ 触发重注册。
    Unauthorized,
    /// 网络/读超时等错误（含 45s chunk 超时检出的半开连接）。
    Failed(pairing_client::PairingClientError),
}

async fn run_supervisor(
    ctx: Arc<AnswerContext>,
    config: Arc<RwLock<DropVoiceConfig>>,
    mut token_rx: watch::Receiver<Option<String>>,
    token_bus: TokenBus,
    get_ip: GetIpFn,
    mut stop_rx: watch::Receiver<bool>,
) {
    let mut backoff = Duration::from_secs(1);

    loop {
        if *stop_rx.borrow_and_update() {
            break;
        }

        // 等待首个 token（heartbeat 注册成功前此处阻塞）。
        let Some(token) = wait_for_token(&mut token_rx, &mut stop_rx).await else {
            break;
        };

        match run_sse_once(&ctx, &config, &token, &mut token_rx, &mut stop_rx).await {
            StreamExit::Stopped => break,
            StreamExit::Ended => {
                // 服务器生命周期关闭：按 retry 提示 ~1s 后重连。
                info!("SSE stream ended (server lifetime); reconnecting");
                backoff = Duration::from_secs(1);
                if sleep_or_stop(SERVER_RETRY_HINT, &mut stop_rx).await {
                    break;
                }
            }
            StreamExit::TokenChanged => {
                info!("pairing_token refreshed; reconnecting SSE");
                backoff = Duration::from_secs(1);
            }
            StreamExit::Unauthorized => {
                warn!("SSE unauthorized (401); refreshing pairing token");
                backoff = Duration::from_secs(1);
                if let Err(e) = heartbeat::refresh_pairing_token(
                    config.clone(),
                    get_ip.clone(),
                    token_bus.clone(),
                )
                .await
                {
                    warn!(error = %e, "pairing token refresh failed");
                    if sleep_or_stop(backoff, &mut stop_rx).await {
                        break;
                    }
                    backoff = (backoff * 2).min(RECONNECT_BACKOFF_MAX);
                }
                // 刷新成功 → token_rx 已收到新值，下一轮循环立即用新 token 建连。
            }
            StreamExit::Failed(e) => {
                warn!(error = %e, "SSE stream failed; reconnecting with backoff");
                if sleep_or_stop(backoff, &mut stop_rx).await {
                    break;
                }
                backoff = (backoff * 2).min(RECONNECT_BACKOFF_MAX);
            }
        }
    }

    // 收尾：关闭全部应答会话（注销客户端 + 关闭 PC）。
    ctx.close_all_sessions().await;
}

/// 等待 pairing_token 出现（None → 阻塞等 watch 变更）。stop 时返回 None。
async fn wait_for_token(
    token_rx: &mut watch::Receiver<Option<String>>,
    stop_rx: &mut watch::Receiver<bool>,
) -> Option<String> {
    loop {
        if let Some(t) = token_rx.borrow_and_update().clone() {
            return Some(t);
        }
        tokio::select! {
            changed = token_rx.changed() => {
                if changed.is_err() {
                    // 总线发送端全部 drop（heartbeat 停止）。
                    return None;
                }
            }
            changed = stop_rx.changed() => {
                if changed.is_err() || *stop_rx.borrow() {
                    return None;
                }
            }
        }
    }
}

/// 退避睡眠；stop 触发时提前返回 true。
async fn sleep_or_stop(d: Duration, stop_rx: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(d) => false,
        changed = stop_rx.changed() => changed.is_err() || *stop_rx.borrow(),
    }
}

/// 建立 SSE 并消费事件直到退出条件满足。
async fn run_sse_once(
    ctx: &Arc<AnswerContext>,
    config: &Arc<RwLock<DropVoiceConfig>>,
    token: &str,
    token_rx: &mut watch::Receiver<Option<String>>,
    stop_rx: &mut watch::Receiver<bool>,
) -> StreamExit {
    let (base_url, device_id) = {
        let cfg = config.read().await;
        let base = pairing_client::resolve_base_url(&cfg.network.pairing_server_url);
        (base, cfg.device.device_id.clone())
    };
    let device_id = match uuid::Uuid::parse_str(&device_id) {
        Ok(id) => id,
        Err(e) => {
            // 配置性错误：退避循环里持续重试没有意义，但保持与 heartbeat 一致
            // 的语义（它对同样错误 return None 停止）。此处按 Failed 处理并
            // 依赖退避上限限频，等待配置被修正。
            warn!(error = %e, "invalid device_id in config");
            return StreamExit::Failed(pairing_client::PairingClientError::Server {
                status: 0,
                code: "INVALID_CONFIG".into(),
                message: format!("invalid device_id: {e}"),
            });
        }
    };

    // Bearer 换一次性票据（长效 token 不进 URL/访问日志），再建 SSE。
    // 票据 401 与 SSE 401 同样归入 Unauthorized → 触发重注册。
    let ticket = match pairing_client::fetch_ticket(&base_url, device_id, token).await {
        Ok(t) => t,
        Err(pairing_client::PairingClientError::Server { status: 401, .. }) => {
            return StreamExit::Unauthorized;
        }
        Err(e) => return StreamExit::Failed(e),
    };

    let stream = match pairing_client::subscribe_events(&base_url, device_id, &ticket).await {
        Ok(s) => s,
        Err(pairing_client::PairingClientError::Server { status: 401, .. }) => {
            return StreamExit::Unauthorized;
        }
        Err(e) => return StreamExit::Failed(e),
    };

    let mut events = EventStream::new(stream);

    loop {
        tokio::select! {
            biased;
            changed = stop_rx.changed() => {
                if changed.is_err() || *stop_rx.borrow() {
                    return StreamExit::Stopped;
                }
            }
            changed = token_rx.changed() => {
                if changed.is_ok()
                    && token_rx.borrow_and_update().clone().as_deref() != Some(token)
                {
                    return StreamExit::TokenChanged;
                }
                // 同值变更（心跳刷新覆盖）→ 继续。
            }
            ev = events.next() => match ev {
                Some(Ok(event)) => {
                    if event.event == "offer" {
                        match serde_json::from_str::<SseOfferEvent>(&event.data) {
                            Ok(offer) => {
                                let endpoint = AnswerEndpoint {
                                    base_url: base_url.clone(),
                                    device_id,
                                    token: token.to_string(),
                                };
                                webrtc::spawn_offer_session(ctx.clone(), endpoint, offer);
                            }
                            Err(e) => {
                                warn!(error = %e, "failed to parse SSE offer event");
                            }
                        }
                    }
                    // ping / keepalive / retry 提示：解析层已消费；连接活性由
                    // chunk 到达本身保证（45s read_timeout）。
                }
                Some(Err(e)) => {
                    // eventsource-stream 的 Transport 变体携带底层 reqwest 错误；
                    // Parser 变体（协议解析）按网络类失败处理，走退避重建。
                    return StreamExit::Failed(match e {
                        eventsource_stream::EventStreamError::Transport(e) => {
                            pairing_client::PairingClientError::Http(e)
                        }
                        // Parser / Utf8（协议解析类失败）：按网络类失败处理，走退避重建。
                        e => pairing_client::PairingClientError::Server {
                            status: 0,
                            code: "SSE_PARSE_FAILED".into(),
                            message: e.to_string(),
                        },
                    });
                }
                None => return StreamExit::Ended,
            }
        }
    }
}

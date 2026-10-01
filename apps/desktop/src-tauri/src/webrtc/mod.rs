//! WebRTC 应答面 —— 桌面端常驻子系统（自 webview 版 `webrtc.ts`/`useWebRTC.ts`
//! 迁入，语义逐条镜像，§5.5/§5.6/§6.3）。
//!
//! 归 Rust 进程所有的理由：应答器需要与 OS 进程同生命周期（托盘隐藏、系统
//! 休眠唤醒都不中断），而 webview 的 JS 定时器/网络会随窗口可见性被
//! WebView2 冻结——这正是"休眠唤醒后手机永久卡在连接中"的根因。应答器的
//! 全部协作者（凭据校验、令牌签发、注入队列）本就在 Rust，迁入后链路内联。
//!
//! 职责：
//! - 每个 incoming offer 一个 `RTCPeerConnection`（空 iceServers，§5.5）
//! - offer → setRemoteDescription → createAnswer → setLocalDescription
//!   → 等 ICE 收集（2s 超时 fallback）→ 返回 answer SDP（全程超时包裹）
//! - DataChannel 协议（§6.3）：`text` → 注入队列 + `ack`；`hello` → 稳定身份
//!   重键；首次配对（code 命中）open 后签发 `token` 下发
//! - 生命周期：`failed/closed` 立即清理；`disconnected` 10s 宽限后清理

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{info, warn};
use uuid::Uuid;
use webrtc::api::{APIBuilder, API};
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::commands::ClientEvent;
use crate::connection::{ConnectionState, CredentialKind};
use crate::error::{AppError, AppResult};
use crate::network::pairing_client::{self, AnswerDecision, SseOfferEvent};
use crate::text::validate_text;

/// SDP 协商单步超时（createAnswer 链路全步骤包裹——旧 JS 版无超时，是潜伏的
/// 悬挂点：PC 异常时 promise 可能永不 settle）。
const NEGOTIATE_STEP_TIMEOUT: Duration = Duration::from_secs(10);

/// ICE 收集超时 fallback（§5.5 2s，与手机端一致）。
const ICE_GATHERING_TIMEOUT: Duration = Duration::from_secs(2);

/// `disconnected` 后注销的宽限期：网络闪断可在这段时间内恢复。
const DISCONNECT_GRACE: Duration = Duration::from_secs(10);

/// DataChannel 应用层消息协议（§6.3，与旧 webview JS 版逐字节兼容）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChannelMessage {
    Text {
        text: String,
    },
    Ack,
    Token {
        token: String,
    },
    Error {
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Hello {
        #[serde(rename = "clientId")]
        client_id: String,
    },
}

/// answer 回填端点（会话任务按 offer 到达时刻的快照使用）。
#[derive(Debug, Clone)]
pub struct AnswerEndpoint {
    pub base_url: String,
    pub device_id: Uuid,
    pub token: String,
}

/// 会话注册表条目：PC + 注册键（初始 session_id，手机 `hello` 后改写为稳定
/// clientId）+ 断连宽限定时器。
struct SessionEntry {
    pc: Arc<RTCPeerConnection>,
    reg_id: Arc<RwLock<String>>,
    grace: Option<JoinHandle<()>>,
}

/// 活跃应答会话注册表（session_id → 条目）。
///
/// `closed` 置位后拒绝新会话（stop_server 与迟到 offer 的竞态由它裁决）。
struct AnswerSessions {
    closed: AtomicBool,
    inner: Mutex<HashMap<String, SessionEntry>>,
}

impl AnswerSessions {
    fn new() -> Self {
        Self {
            closed: AtomicBool::new(false),
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// 注册会话；已关闭（stop 流程中）则拒绝，调用方负责关闭 PC。
    async fn insert(
        &self,
        session_id: &str,
        pc: Arc<RTCPeerConnection>,
        reg_id: Arc<RwLock<String>>,
    ) -> Result<()> {
        let mut inner = self.inner.lock().await;
        if self.closed.load(Ordering::SeqCst) {
            return Err(anyhow!("answer sessions closed"));
        }
        inner.insert(
            session_id.to_string(),
            SessionEntry {
                pc,
                reg_id,
                grace: None,
            },
        );
        Ok(())
    }

    /// 取出并移除会话（取消宽限定时器）。返回 PC 供调用方关闭。
    async fn remove(
        &self,
        session_id: &str,
    ) -> Option<(Arc<RTCPeerConnection>, Arc<RwLock<String>>)> {
        let mut inner = self.inner.lock().await;
        inner.remove(session_id).map(|e| {
            if let Some(g) = e.grace {
                g.abort();
            }
            (e.pc, e.reg_id)
        })
    }

    /// `disconnected` 时挂宽限定时器（覆盖旧定时器）。
    async fn set_grace(&self, session_id: &str, grace: JoinHandle<()>) {
        let mut inner = self.inner.lock().await;
        if let Some(entry) = inner.get_mut(session_id) {
            if let Some(old) = entry.grace.take() {
                old.abort();
            }
            entry.grace = Some(grace);
        }
    }

    /// `connected` 时取消宽限定时器。
    async fn cancel_grace(&self, session_id: &str) {
        let mut inner = self.inner.lock().await;
        if let Some(entry) = inner.get_mut(session_id) {
            if let Some(g) = entry.grace.take() {
                g.abort();
            }
        }
    }

    /// 停机收尾：置 closed、清空注册表、注销全部客户端并关闭全部 PC。
    async fn close_all(&self, ctx: &AnswerContext) {
        self.closed.store(true, Ordering::SeqCst);
        let entries: Vec<SessionEntry> = {
            let mut inner = self.inner.lock().await;
            inner.drain().map(|(_, e)| e).collect()
        };
        for entry in entries {
            if let Some(g) = entry.grace {
                g.abort();
            }
            let reg_id = entry.reg_id.read().await.clone();
            unregister_client(ctx, &reg_id).await;
            if let Err(e) = entry.pc.close().await {
                warn!(error = %e, "failed to close peer connection during shutdown");
            }
        }
    }
}

/// 应答器共享上下文（信令监督任务创建一份，逐会话/逐回调克隆 Arc）。
///
/// UI 通知经 `event_sink` 闭包发出（生产实现包一层 `app.emit`），使应答器
/// 核心可在无 Tauri 运行时的单测中构造。
pub struct AnswerContext {
    event_sink: ClientEventSink,
    connection_state: Arc<Mutex<Option<ConnectionState>>>,
    sessions: Arc<AnswerSessions>,
    api: Arc<API>,
}

/// `client_registered` UI 事件出口。
pub type ClientEventSink = Arc<dyn Fn(ClientEvent) + Send + Sync>;

impl AnswerContext {
    pub fn new(app: AppHandle, connection_state: Arc<Mutex<Option<ConnectionState>>>) -> Self {
        let sink: ClientEventSink = Arc::new(move |ev: ClientEvent| {
            if let Err(e) = app.emit("client_registered", ev) {
                warn!(error = %e, "failed to emit client_registered event");
            }
        });
        Self::with_event_sink(connection_state, sink)
    }

    /// 测试/嵌入入口：自定义事件出口。
    pub fn with_event_sink(
        connection_state: Arc<Mutex<Option<ConnectionState>>>,
        event_sink: ClientEventSink,
    ) -> Self {
        // 空 MediaEngine：纯 DataChannel 应答，无音视频 m-line 需要编解码注册。
        // 空 iceServers（§5.5）：同 LAN host 候选直连，无 STUN/TURN。
        // ICE mDNS 默认 QueryOnly：接受并解析手机（Chromium）的 .local 候选，
        // 本地候选通告明文 IP。
        let api = APIBuilder::new().build();
        Self {
            event_sink,
            connection_state,
            sessions: Arc::new(AnswerSessions::new()),
            api: Arc::new(api),
        }
    }

    /// 停机收尾（信令监督任务退出时调用）。
    pub async fn close_all_sessions(&self) {
        self.sessions.close_all(self).await;
    }
}

/// 处理一条 incoming offer：校验凭据 → 协商 → 回填 answer → 接线 DataChannel。
///
/// 独立任务运行（由信令监督任务 spawn），不阻塞 SSE 循环；任一步失败即
/// 丢弃该会话（手机侧 30s 长轮询超时后自行退避重试）。
pub fn spawn_offer_session(
    ctx: Arc<AnswerContext>,
    endpoint: AnswerEndpoint,
    offer: SseOfferEvent,
) {
    tokio::spawn(async move {
        if let Err(e) = run_offer_session(&ctx, &endpoint, &offer).await {
            warn!(session_id = %offer.session_id, error = %e, "offer session failed");
        }
    });
}

async fn run_offer_session(
    ctx: &Arc<AnswerContext>,
    endpoint: &AnswerEndpoint,
    offer: &SseOfferEvent,
) -> Result<()> {
    // 1. 校验 credential（§3.2/§3.3，桌面本地比对）。服务已停止 → 放弃。
    let kind = {
        let guard = ctx.connection_state.lock().await;
        match guard.as_ref() {
            Some(cs) => cs.validate_credential(&offer.credential).await,
            None => return Err(anyhow!("connection service not running")),
        }
    };
    let kind = match kind {
        CredentialKind::Code | CredentialKind::Token => kind,
        CredentialKind::Invalid => {
            // credential 错误 → rejected（§3.2 安全模型）。
            info!(session_id = %offer.session_id, "offer rejected: invalid credential");
            let _ = submit_answer(
                endpoint,
                &offer.session_id,
                &AnswerDecision::Rejected {
                    reason: "invalid_credential".into(),
                },
            )
            .await;
            return Ok(());
        }
    };

    // 2. 协商 answer。
    let (pc, answer_sdp) = negotiate_answer(ctx, &offer.sdp).await?;

    // 3. 回填 accepted answer。
    submit_answer(
        endpoint,
        &offer.session_id,
        &AnswerDecision::Accepted { sdp: answer_sdp },
    )
    .await?;

    // 4. 接线 DataChannel + 连接状态清理 + 注册会话。
    wire_session(ctx, &offer.session_id, kind, pc).await?;
    Ok(())
}

/// SDP 协商：offer → answer（等 ICE 收集完成，2s fallback，§5.5 非 trickle）。
async fn negotiate_answer(
    ctx: &AnswerContext,
    offer_sdp: &str,
) -> Result<(Arc<RTCPeerConnection>, String)> {
    let pc = ctx
        .api
        .new_peer_connection(RTCConfiguration::default())
        .await?;

    let offer = RTCSessionDescription::offer(offer_sdp.to_string())?;
    tokio::time::timeout(NEGOTIATE_STEP_TIMEOUT, pc.set_remote_description(offer))
        .await
        .map_err(|_| anyhow!("set_remote_description timed out"))??;
    let answer = tokio::time::timeout(NEGOTIATE_STEP_TIMEOUT, pc.create_answer(None))
        .await
        .map_err(|_| anyhow!("create_answer timed out"))??;
    tokio::time::timeout(NEGOTIATE_STEP_TIMEOUT, pc.set_local_description(answer))
        .await
        .map_err(|_| anyhow!("set_local_description timed out"))??;

    // 等 ICE 收集完成（Pion 风格 promise；超时 fallback 用已收集候选继续）。
    let mut gathering_done = pc.gathering_complete_promise().await;
    let _ = tokio::time::timeout(ICE_GATHERING_TIMEOUT, gathering_done.recv()).await;

    let sdp = pc
        .local_description()
        .await
        .map(|d| d.sdp)
        .unwrap_or_default();
    Ok((Arc::new(pc), sdp))
}

/// 接线一个已协商的会话：DataChannel 协议处理 + PC 生命周期清理 + 注册表登记。
async fn wire_session(
    ctx: &Arc<AnswerContext>,
    session_id: &str,
    credential_kind: CredentialKind,
    pc: Arc<RTCPeerConnection>,
) -> Result<()> {
    // 注册键（初始 session_id；手机 hello 后改写为稳定 clientId）。
    let reg_id: Arc<RwLock<String>> = Arc::new(RwLock::new(session_id.to_string()));

    // DataChannel（offerer = 手机创建通道，桌面经 on_data_channel 接收，§5.6）。
    let dc_ctx = ctx.clone();
    let dc_sid = session_id.to_string();
    let dc_reg = reg_id.clone();
    pc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
        let ctx = dc_ctx.clone();
        let sid = dc_sid.clone();
        let reg = dc_reg.clone();
        Box::pin(async move {
            wire_data_channel(&ctx, &sid, credential_kind, reg, dc).await;
        })
    }));

    // PC 生命周期：failed/closed 立即清理；disconnected 10s 宽限；connected 取消宽限。
    let st_ctx = ctx.clone();
    let st_sid = session_id.to_string();
    let st_reg = reg_id.clone();
    pc.on_peer_connection_state_change(Box::new(move |state| {
        let ctx = st_ctx.clone();
        let sid = st_sid.clone();
        let reg = st_reg.clone();
        Box::pin(async move {
            handle_connection_state(&ctx, &sid, reg, state).await;
        })
    }));

    // 登记注册表（stop 竞态 → 拒绝并关闭）。
    if let Err(e) = ctx.sessions.insert(session_id, pc.clone(), reg_id).await {
        warn!(session_id = %session_id, error = %e, "answer sessions closed; dropping session");
        let _ = pc.close().await;
        return Err(e);
    }
    Ok(())
}

/// 处理 PC 连接状态迁移（镜像 useWebRTC.ts 的清理策略）。
async fn handle_connection_state(
    ctx: &Arc<AnswerContext>,
    session_id: &str,
    reg_id: Arc<RwLock<String>>,
    state: RTCPeerConnectionState,
) {
    match state {
        RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed => {
            close_session(ctx, session_id, reg_id).await;
        }
        RTCPeerConnectionState::Disconnected => {
            let grace_ctx = ctx.clone();
            let sid = session_id.to_string();
            let reg = reg_id.clone();
            let grace = tokio::spawn(async move {
                tokio::time::sleep(DISCONNECT_GRACE).await;
                close_session(&grace_ctx, &sid, reg).await;
            });
            ctx.sessions.set_grace(session_id, grace).await;
        }
        RTCPeerConnectionState::Connected => {
            ctx.sessions.cancel_grace(session_id).await;
        }
        _ => {}
    }
}

/// 移除会话：注销客户端 → 关闭 PC（remove 已取消宽限定时器）。
async fn close_session(ctx: &Arc<AnswerContext>, session_id: &str, reg_id: Arc<RwLock<String>>) {
    if let Some((pc, _)) = ctx.sessions.remove(session_id).await {
        let reg = reg_id.read().await.clone();
        unregister_client(ctx, &reg).await;
        if let Err(e) = pc.close().await {
            warn!(session_id = %session_id, error = %e, "failed to close peer connection");
        }
    }
}

/// DataChannel 接线（镜像 webrtc.ts wireDataChannel，§6.3）。
async fn wire_data_channel(
    ctx: &Arc<AnswerContext>,
    session_id: &str,
    credential_kind: CredentialKind,
    reg_id: Arc<RwLock<String>>,
    dc: Arc<RTCDataChannel>,
) {
    info!(session_id = %session_id, "DataChannel received; wiring");

    // open：注册客户端 + emit；首次配对（code 命中）签发 token 下发。
    let open_ctx = ctx.clone();
    let open_reg = reg_id.clone();
    let open_dc = dc.clone();
    dc.on_open(Box::new(move || {
        let ctx = open_ctx;
        let reg = open_reg;
        let dc = open_dc;
        Box::pin(async move {
            let client_id = reg.read().await.clone();
            if let Err(e) = register_client(&ctx, &client_id).await {
                // max_connections 等：拒绝该会话（旧 JS 版仅记日志是漏洞）。
                warn!(client_id = %client_id, error = %e, "register failed; rejecting session");
                let _ = dc.close().await;
                return;
            }
            if credential_kind == CredentialKind::Code {
                let token = {
                    let guard = ctx.connection_state.lock().await;
                    match guard.as_ref() {
                        Some(cs) => cs.issue_connection_token().await,
                        None => return,
                    }
                };
                if let Err(e) = dc_send(&dc, &ChannelMessage::Token { token }).await {
                    warn!(error = %e, "failed to send connection token");
                }
            }
        })
    }));

    // message：hello 重键 / text → 注入队列 + ack。
    let msg_ctx = ctx.clone();
    let msg_reg = reg_id.clone();
    let msg_dc = dc.clone();
    dc.on_message(Box::new(move |msg: DataChannelMessage| {
        let ctx = msg_ctx.clone();
        let reg = msg_reg.clone();
        let dc = msg_dc.clone();
        Box::pin(async move {
            if !msg.is_string {
                warn!("ignoring binary DataChannel message");
                return;
            }
            let parsed: ChannelMessage = match serde_json::from_slice(&msg.data) {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = %e, "DataChannel message parse failed");
                    return;
                }
            };
            match parsed {
                ChannelMessage::Hello { client_id } => {
                    handle_hello(&ctx, reg, &client_id).await;
                }
                ChannelMessage::Text { text } => {
                    handle_text(&ctx, &reg, &dc, text).await;
                }
                // ack/token/error 是手机→桌面方向不该出现的消息；debug 记录。
                other => {
                    tracing::debug!(message = ?other, "ignoring unexpected DataChannel message");
                }
            }
        })
    }));

    // close：注销客户端。
    let close_ctx = ctx.clone();
    let close_reg = reg_id.clone();
    dc.on_close(Box::new(move || {
        let ctx = close_ctx.clone();
        let reg = close_reg.clone();
        Box::pin(async move {
            let client_id = reg.read().await.clone();
            unregister_client(&ctx, &client_id).await;
        })
    }));
}

/// 手机上报稳定身份：注册键 session_id → clientId（§客户端身份）。
async fn handle_hello(ctx: &AnswerContext, reg_id: Arc<RwLock<String>>, client_id: &str) {
    let old = reg_id.read().await.clone();
    if old == client_id {
        return;
    }
    *reg_id.write().await = client_id.to_string();
    info!(from = %old, to = %client_id, "hello: re-keying registration");
    unregister_client(ctx, &old).await;
    if let Err(e) = register_client(ctx, client_id).await {
        warn!(client_id = %client_id, error = %e, "re-register under clientId failed");
    }
}

/// text：校验 → 入注入队列（直调，无 invoke 跳板）→ 回 ack；失败回
/// INJECTION_FAILED（手机侧 toast，不计入连接状态）。
async fn handle_text(
    ctx: &AnswerContext,
    reg_id: &Arc<RwLock<String>>,
    dc: &Arc<RTCDataChannel>,
    text: String,
) {
    let client_id = reg_id.read().await.clone();
    let result: AppResult<()> = {
        let guard = ctx.connection_state.lock().await;
        let Some(cs) = guard.as_ref() else {
            warn!(client_id = %client_id, "connection service not running; dropping text");
            return;
        };
        let max_text_length = cs.config.read().await.injection.max_text_length;
        validate_text(&text, max_text_length).and_then(|()| {
            cs.connection_manager
                .enqueue_injection(client_id.clone(), text.clone())
        })
    };
    match result {
        Ok(()) => {
            info!(client_id = %client_id, chars = text.chars().count(), "text injection enqueued");
            if let Err(e) = dc_send(dc, &ChannelMessage::Ack).await {
                warn!(error = %e, "failed to send ack");
            }
        }
        Err(e) => {
            warn!(client_id = %client_id, error = %e, "injection rejected");
            let msg = ChannelMessage::Error {
                code: "INJECTION_FAILED".into(),
                message: Some(e.to_string()),
            };
            if let Err(e) = dc_send(dc, &msg).await {
                warn!(error = %e, "failed to send INJECTION_FAILED");
            }
        }
    }
}

/// 注册客户端（DataChannel open / hello 重键后调用）+ emit `client_registered`。
async fn register_client(ctx: &AnswerContext, client_id: &str) -> AppResult<()> {
    {
        let guard = ctx.connection_state.lock().await;
        let cs = guard.as_ref().ok_or(AppError::ServerStartFailed {
            reason: "connection service not running".into(),
        })?;
        cs.connection_manager
            .register(client_id.to_string())
            .await?;
    }
    info!(client_id = %client_id, "client registered (DataChannel open)");
    (ctx.event_sink)(ClientEvent {
        client_id: client_id.into(),
    });
    Ok(())
}

/// 注销客户端（DataChannel close / 会话清理时调用）。
async fn unregister_client(ctx: &AnswerContext, client_id: &str) {
    {
        let guard = ctx.connection_state.lock().await;
        if let Some(cs) = guard.as_ref() {
            cs.connection_manager.unregister(client_id).await;
        }
    }
    info!(client_id = %client_id, "client unregistered (DataChannel closed)");
}

/// 发送 DataChannel 消息（JSON 序列化 + send_text）。
async fn dc_send(dc: &Arc<RTCDataChannel>, msg: &ChannelMessage) -> Result<()> {
    let payload = serde_json::to_string(msg)?;
    dc.send_text(payload)
        .await
        .map_err(|e| anyhow!("datachannel send failed: {e}"))?;
    Ok(())
}

/// 回填 answer（薄封装，统一错误路径）。
async fn submit_answer(
    endpoint: &AnswerEndpoint,
    session_id: &str,
    decision: &AnswerDecision,
) -> Result<()> {
    pairing_client::submit_answer(
        &endpoint.base_url,
        endpoint.device_id,
        &endpoint.token,
        session_id,
        decision,
    )
    .await
    .map_err(|e| anyhow!("submit answer failed: {e}"))
}

#[cfg(test)]
mod tests;

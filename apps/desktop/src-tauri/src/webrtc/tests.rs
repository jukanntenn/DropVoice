//! 应答面回环测试：进程内 webrtc-rs offerer（模拟手机）↔ Rust 应答器。
//!
//! 直接交换 SDP（不经 HTTP/SSE），验证旧架构在浏览器里才能覆盖的完整链路：
//! 协商 → DataChannel 建立 → hello 重键 → text 入注入队列（MockInjector）→
//! ack → 首配 token 签发下发。

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::{negotiate_answer, wire_session, AnswerContext, ChannelMessage, ClientEventSink};
use crate::config::DropVoiceConfig;
use crate::connection::{ConnectionState, CredentialKind};
use crate::text::MockInjector;

/// 回环各步骤的统一超时（进程内 ICE/DTLS 通常秒级完成，给足余量）。
const TEST_TIMEOUT: Duration = Duration::from_secs(20);

/// 构建带 MockInjector 的 ConnectionState（注入队列处理任务 delay=0）。
fn test_state() -> (Arc<MockInjector>, Arc<Mutex<Option<ConnectionState>>>) {
    let mut config = DropVoiceConfig::default();
    config.server.max_connections = 5;
    config.injection.queue_size = 100;
    let injector = Arc::new(MockInjector::new());
    let cs = ConnectionState::with_injector(config, injector.clone());
    let cm = cs.connection_manager.clone();
    let metrics = cs.metrics.clone();
    cm.start_processing_task(0, metrics, None);
    (injector, Arc::new(Mutex::new(Some(cs))))
}

/// 手机侧（offerer）收到的事件记录。
#[derive(Clone, Default)]
struct PhoneLog {
    opened: Arc<std::sync::Mutex<bool>>,
    messages: Arc<std::sync::Mutex<Vec<ChannelMessage>>>,
}

/// 构建手机侧 offerer PC + DataChannel + 消息记录。
async fn build_offerer() -> anyhow::Result<(
    Arc<webrtc::peer_connection::RTCPeerConnection>,
    Arc<RTCDataChannel>,
    PhoneLog,
)> {
    let api = APIBuilder::new().build();
    let offerer = Arc::new(api.new_peer_connection(RTCConfiguration::default()).await?);

    // 手机是 offerer：本侧 createDataChannel（label `dropvoice`，ordered，§6.2）。
    let dc = offerer
        .create_data_channel("dropvoice", Some(Default::default()))
        .await?;

    let log = PhoneLog::default();
    let opened = log.opened.clone();
    dc.on_open(Box::new(move || {
        let mut o = opened.lock().unwrap();
        *o = true;
        Box::pin(async {})
    }));
    let messages = log.messages.clone();
    dc.on_message(Box::new(move |msg| {
        if msg.is_string {
            if let Ok(m) = serde_json::from_slice::<ChannelMessage>(&msg.data) {
                messages.lock().unwrap().push(m);
            }
        }
        Box::pin(async {})
    }));

    Ok((offerer, dc, log))
}

/// 等 offerer DataChannel open。
async fn wait_open(log: &PhoneLog) {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    while !*log.opened.lock().unwrap() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "DataChannel never opened"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// 轮询断言（带超时）。
async fn eventually<F: Fn() -> bool>(cond: F, what: &str) {
    let deadline = tokio::time::Instant::now() + TEST_TIMEOUT;
    while !cond() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "timeout waiting for: {what}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// 完整回环：offer（含候选）→ 应答器协商+接线 → answer → DataChannel 通信。
#[tokio::test]
async fn offerer_answerer_round_trip() {
    let (injector, cs_holder) = test_state();

    // 签发配对码（首配 credential）。
    let code = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().issue_pairing_code().await
    };

    // registered 事件记录（应答器 → UI 出口）。
    let registered: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(vec![]));
    let sink: ClientEventSink = {
        let r = registered.clone();
        Arc::new(move |ev: crate::commands::ClientEvent| r.lock().unwrap().push(ev.client_id))
    };
    let ctx = Arc::new(AnswerContext::with_event_sink(cs_holder.clone(), sink));

    // 手机侧 offerer。
    let (offerer, dc, log) = build_offerer().await.unwrap();

    // offer + 等候选收集（非 trickle，§5.5）。
    let offer = offerer.create_offer(None).await.unwrap();
    offerer.set_local_description(offer).await.unwrap();
    let mut gathered = offerer.gathering_complete_promise().await;
    let _ = tokio::time::timeout(Duration::from_secs(5), gathered.recv()).await;
    let offer_sdp = offerer.local_description().await.unwrap().sdp;

    // credential 校验：code 命中（与 run_offer_session 相同的判定）。
    let kind = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().validate_credential(&code).await
    };
    assert_eq!(kind, CredentialKind::Code);

    // 应答器协商 + 接线（HTTP 回填不在回环路径）。
    let (pc, answer_sdp) = negotiate_answer(&ctx, &offer_sdp).await.unwrap();
    wire_session(&ctx, "sess-1", kind, pc).await.unwrap();
    assert!(!answer_sdp.is_empty());

    // 手机应用 answer → ICE/DTLS → DataChannel open。
    offerer
        .set_remote_description(RTCSessionDescription::answer(answer_sdp).unwrap())
        .await
        .unwrap();
    wait_open(&log).await;

    // hello：注册键 session_id → 稳定 clientId。
    dc.send_text(r#"{"type":"hello","clientId":"phone-1"}"#)
        .await
        .unwrap();
    eventually(
        || {
            let r = registered.lock().unwrap();
            r.len() >= 2 && r[0] == "sess-1" && r[1] == "phone-1"
        },
        "hello re-key registration (sess-1 then phone-1)",
    )
    .await;

    // text → MockInjector 收到 + ack 回到手机。
    dc.send_text(r#"{"type":"text","text":"你好 dropvoice"}"#)
        .await
        .unwrap();
    eventually(
        || {
            injector
                .recorded()
                .iter()
                .any(|(t, _, _)| t == "你好 dropvoice")
        },
        "text reaches MockInjector",
    )
    .await;
    eventually(
        || {
            log.messages
                .lock()
                .unwrap()
                .iter()
                .any(|m| matches!(m, ChannelMessage::Ack))
        },
        "ack reaches offerer",
    )
    .await;

    // 首配（code 命中）→ token 下发（dvct_ 前缀）。
    eventually(
        || {
            log.messages
                .lock()
                .unwrap()
                .iter()
                .any(|m| matches!(m, ChannelMessage::Token { token } if token.starts_with("dvct_")))
        },
        "connection token issued to offerer",
    )
    .await;

    // 活跃客户端 = phone-1（重键后）。
    let count = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().connection_manager.count().await
    };
    assert_eq!(count, 1);

    // 停机收尾：客户端注销、会话清空。
    ctx.close_all_sessions().await;
    let count = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().connection_manager.count().await
    };
    assert_eq!(count, 0);
}

/// token credential 命中（重连路径）。
#[tokio::test]
async fn token_credential_validates_after_issuance() {
    let (_injector, cs_holder) = test_state();
    let token = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().issue_connection_token().await
    };
    assert!(token.starts_with("dvct_"));
    let kind = {
        let guard = cs_holder.lock().await;
        guard.as_ref().unwrap().validate_credential(&token).await
    };
    assert_eq!(kind, CredentialKind::Token);
}

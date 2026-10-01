//! 信令会话存储 + SSE offer 推送桥（webrtc-scan-direct-design §4.6）。
//!
//! `SignalStore` 持有：
//! - `sessions`：`session_id -> SignalSession`，TTL 60s，cleanup_expired 周期回收。
//! - `offers`：`device_id -> mpsc::Sender<OfferEvent>`，SSE 长连接订阅时注册，
//!   POST offer 时推送。这是决策点①：用 per-device mpsc 桥接 POST offer → SSE 推送。
//!
//! 长轮询唤醒用 `tokio::sync::Notify`，采用双检模式（§4.6：先查共享状态再 await Notify）。
//! `Notify` 存储一个 permit（notify.rs:29-46），但仅存一个（多次 notify_one 合并为 1），
//! 因此双检模式是必要的——确保 answer 就绪后即使 notify 已合并，查询也能拿到结果。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Notify, RwLock};
use uuid::Uuid;

use crate::config::{MAX_SESSIONS_PER_DEVICE, SIGNAL_SESSION_TTL, SSE_TICKET_TTL};

/// 桌面回填的 answer 状态。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerStatus {
    /// 已接受（附带 SDP）。
    Accepted { sdp: String },
    /// 已拒绝（附带原因，如 invalid_credential）。
    Rejected { reason: String },
}

/// 信令会话（§4.6）。
pub struct SignalSession {
    pub session_id: String,
    pub device_id: Uuid,
    pub offer_sdp: String,
    pub credential: String,
    pub answer: Option<AnswerStatus>,
    pub created_at: Instant,
    /// 唤醒手机长轮询。Notify 存储一个 permit，双检模式下保证唤醒可靠。
    pub notify: Arc<Notify>,
}

impl SignalSession {
    pub fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.created_at) >= SIGNAL_SESSION_TTL
    }
}

/// SSE 推送给桌面的事件。
#[derive(Debug, Clone)]
pub struct OfferEvent {
    pub session_id: String,
    pub credential: String,
    pub sdp: String,
}

/// SSE 订阅句柄。SSE handler 持有 rx + sub_id，退出时带 sub_id 反注册
///（identity-based compare-and-delete，§7：防止老任务误删新订阅）。
pub struct OfferSubscription {
    pub rx: mpsc::Receiver<OfferEvent>,
    /// 该订阅的身份标识；unsubscribe 时只删匹配自己 sub_id 的条目。
    pub sub_id: u64,
}

/// 全局单调递增的订阅身份计数器（§7）。
static NEXT_SUB_ID: AtomicU64 = AtomicU64::new(1);

/// Per-device SSE 订阅表：`device_id -> (offer 推送 sender, 订阅身份 sub_id)`。
type OfferMap = Arc<RwLock<HashMap<Uuid, (mpsc::Sender<OfferEvent>, u64)>>>;

/// SSE 订阅票据（一次性，短 TTL）。
#[derive(Debug)]
struct TicketEntry {
    device_id: Uuid,
    expires_at: Instant,
}

impl TicketEntry {
    fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

/// 信令会话存储 + offer 推送桥 + SSE 订阅票据。
#[derive(Clone)]
pub struct SignalStore {
    sessions: Arc<RwLock<HashMap<String, SignalSession>>>,
    /// `device_id -> (sender, sub_id)`：sub_id 让 unsubscribe 做身份化 compare-and-delete。
    offers: OfferMap,
    /// 待兑换的 SSE 订阅票据（`ticket -> (device_id, expires_at)`）。
    /// 长效 pairing_token 不进 URL（Caddy 日志记录完整 uri），票据单次使用、
    /// 60s 过期，出现在日志中无害。
    tickets: Arc<RwLock<HashMap<String, TicketEntry>>>,
}

impl SignalStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            offers: Arc::new(RwLock::new(HashMap::new())),
            tickets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 签发 SSE 订阅票据（Bearer 校验通过后调用）。顺带清扫过期票据。
    pub async fn issue_ticket(&self, device_id: Uuid) -> String {
        let ticket = crate::api::error::generate_token();
        let now = Instant::now();
        let mut tickets = self.tickets.write().await;
        tickets.retain(|_, e| !e.is_expired(now));
        tickets.insert(
            ticket.clone(),
            TicketEntry {
                device_id,
                expires_at: now + SSE_TICKET_TTL,
            },
        );
        ticket
    }

    /// 兑换 SSE 订阅票据：单次使用、限期、device 必须匹配。
    /// 返回 false 表示票据无效（含已用/过期/设备不匹配）。
    pub async fn redeem_ticket(&self, ticket: &str, device_id: Uuid) -> bool {
        let now = Instant::now();
        let mut tickets = self.tickets.write().await;
        match tickets.remove(ticket) {
            // remove 即单次使用：并发兑换同一票据只有一个成功。
            Some(entry) => !entry.is_expired(now) && entry.device_id == device_id,
            None => false,
        }
    }

    /// 创建会话。返回 Err(TooManySessions) 当该 device 并发会话超上限（§4.6）。
    pub async fn create_session(
        &self,
        device_id: Uuid,
        offer_sdp: String,
        credential: String,
    ) -> Result<String, CreateSessionError> {
        let mut sessions = self.sessions.write().await;
        // 计数该 device 的活跃会话（含未过期）。
        let now = Instant::now();
        let active = sessions
            .values()
            .filter(|s| s.device_id == device_id && !s.is_expired(now))
            .count();
        if active >= MAX_SESSIONS_PER_DEVICE {
            return Err(CreateSessionError::TooManySessions);
        }
        let session_id = Uuid::new_v4().to_string();
        sessions.insert(
            session_id.clone(),
            SignalSession {
                session_id: session_id.clone(),
                device_id,
                offer_sdp,
                credential,
                answer: None,
                created_at: now,
                notify: Arc::new(Notify::new()),
            },
        );
        Ok(session_id)
    }

    /// 桌面回填 answer，唤醒手机长轮询。
    /// 返回 Err(SessionNotFound) 当会话不存在/已过期。
    pub async fn submit_answer(
        &self,
        session_id: &str,
        answer: AnswerStatus,
    ) -> Result<(), SubmitAnswerError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or(SubmitAnswerError::NotFound)?;
        if session.is_expired(Instant::now()) {
            sessions.remove(session_id);
            return Err(SubmitAnswerError::NotFound);
        }
        session.answer = Some(answer);
        // 唤醒等待的长轮询（permit 存储一个，合并安全）。
        session.notify.notify_waiters();
        Ok(())
    }

    /// 手机长轮询：检查 answer 状态。双检模式（§4.6）。
    /// 返回：
    /// - Ok(Some) → answer 就绪
    /// - Ok(None) → answer 未就绪，调用方应 await Notify（最多 hold 30s）
    /// - Err(NotFound) → 会话不存在/已过期
    ///
    /// 返回的 `Option<Arc<Notify>>` 供调用方 await（持锁期间取出，避免 TOCTOU）。
    pub async fn peek_answer(
        &self,
        session_id: &str,
    ) -> Result<(Option<AnswerStatus>, Option<Arc<Notify>>), NotFound> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or(NotFound)?;
        if session.is_expired(Instant::now()) {
            return Err(NotFound);
        }
        Ok((session.answer.clone(), Some(session.notify.clone())))
    }

    /// 注册 SSE 订阅。返回带 sub_id 的句柄。若该 device 已有订阅则替换
    ///（每桌面维持 1 条 SSE）；新订阅拿到唯一 sub_id，老订阅退出时带自己的
    /// sub_id 做 compare-and-delete，不会误删新订阅（§7 根因修复）。
    pub async fn subscribe_offers(&self, device_id: Uuid) -> OfferSubscription {
        let sub_id = NEXT_SUB_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel::<OfferEvent>(16);
        self.offers.write().await.insert(device_id, (tx, sub_id));
        OfferSubscription { rx, sub_id }
    }

    /// 向桌面的 SSE 推送 offer。若桌面未订阅（SSE 未就绪/离线）返回 false。
    pub async fn push_offer(&self, session: &SignalSession) -> bool {
        let offers = self.offers.read().await;
        let Some((tx, _)) = offers.get(&session.device_id) else {
            return false;
        };
        let event = OfferEvent {
            session_id: session.session_id.clone(),
            credential: session.credential.clone(),
            sdp: session.offer_sdp.clone(),
        };
        // try_send：SSE 消费慢导致满则丢弃（不应发生，16 缓冲足够）。
        tx.try_send(event).is_ok()
    }

    /// 身份化移除 SSE 订阅：仅当当前条目的 sub_id 与传入一致才删（§7）。
    /// 这样老 SSE 任务退出时（持有过期 sub_id）不会删掉已被新订阅替换的条目。
    pub async fn unsubscribe_offers(&self, device_id: Uuid, sub_id: u64) {
        let mut offers = self.offers.write().await;
        if offers
            .get(&device_id)
            .map(|(_, id)| *id)
            .is_some_and(|id| id == sub_id)
        {
            offers.remove(&device_id);
        }
    }

    /// 当前是否有人订阅该 device 的 offer（测试/诊断用）。
    #[cfg(test)]
    pub async fn has_subscriber(&self, device_id: Uuid) -> bool {
        self.offers.read().await.contains_key(&device_id)
    }

    /// 按 session_id 获取会话（用于 POST offer 后推送）。
    pub async fn get_session(&self, session_id: &str) -> Option<SignalSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// GC：移除过期会话。返回移除数。
    pub fn cleanup_expired_sessions(&self) -> usize {
        // try_write 避免阻塞后台任务（若正在被写锁占用则跳过本次）。
        let mut sessions = match self.sessions.try_write() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let now = Instant::now();
        let before = sessions.len();
        sessions.retain(|_, s| !s.is_expired(now));
        before - sessions.len()
    }

    /// 当前活跃（未过期）会话数（L3 指标）。
    pub fn active_session_count(&self) -> usize {
        let sessions = match self.sessions.try_read() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let now = Instant::now();
        sessions.values().filter(|s| !s.is_expired(now)).count()
    }
}

impl Default for SignalStore {
    fn default() -> Self {
        Self::new()
    }
}

/// SignalSession 需要被 clone（get_session 返回 owned，push_offer 借用）。
impl Clone for SignalSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            device_id: self.device_id,
            offer_sdp: self.offer_sdp.clone(),
            credential: self.credential.clone(),
            answer: self.answer.clone(),
            created_at: self.created_at,
            notify: self.notify.clone(),
        }
    }
}

#[derive(Debug)]
pub enum CreateSessionError {
    TooManySessions,
}

#[derive(Debug)]
pub enum SubmitAnswerError {
    NotFound,
}

#[derive(Debug)]
pub struct NotFound;

/// 显式标记未使用（Duration 来自 hold 超时计算，保留导入供未来用）。
#[allow(dead_code)]
fn _unused_duration() -> Duration {
    Duration::from_secs(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> SignalStore {
        SignalStore::new()
    }

    #[tokio::test]
    async fn create_and_peek_session() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "offer-sdp".into(), "123456".into())
            .await
            .unwrap();
        // answer 未就绪。
        let (answer, notify) = store.peek_answer(&sid).await.unwrap();
        assert!(answer.is_none());
        assert!(notify.is_some());
    }

    #[tokio::test]
    async fn submit_answer_then_peek_returns_answer() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "offer-sdp".into(), "123456".into())
            .await
            .unwrap();
        store
            .submit_answer(
                &sid,
                AnswerStatus::Accepted {
                    sdp: "answer-sdp".into(),
                },
            )
            .await
            .unwrap();
        let (answer, _) = store.peek_answer(&sid).await.unwrap();
        match answer.unwrap() {
            AnswerStatus::Accepted { sdp } => assert_eq!(sdp, "answer-sdp"),
            AnswerStatus::Rejected { .. } => panic!("expected Accepted"),
        }
    }

    #[tokio::test]
    async fn submit_answer_rejected() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "offer-sdp".into(), "123456".into())
            .await
            .unwrap();
        store
            .submit_answer(
                &sid,
                AnswerStatus::Rejected {
                    reason: "invalid_credential".into(),
                },
            )
            .await
            .unwrap();
        let (answer, _) = store.peek_answer(&sid).await.unwrap();
        match answer.unwrap() {
            AnswerStatus::Rejected { reason } => assert_eq!(reason, "invalid_credential"),
            AnswerStatus::Accepted { .. } => panic!("expected Rejected"),
        }
    }

    #[tokio::test]
    async fn peek_nonexistent_session_returns_not_found() {
        let store = make_store();
        let result = store.peek_answer("nonexistent").await;
        assert!(matches!(result, Err(NotFound)));
    }

    #[tokio::test]
    async fn submit_answer_nonexistent_returns_not_found() {
        let store = make_store();
        let result = store
            .submit_answer("nonexistent", AnswerStatus::Accepted { sdp: "x".into() })
            .await;
        assert!(matches!(result, Err(SubmitAnswerError::NotFound)));
    }

    /// 双检模式唤醒：notify 先发后 await 仍能唤醒。
    #[tokio::test]
    async fn long_poll_notify_wakes_after_answer() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "offer-sdp".into(), "123456".into())
            .await
            .unwrap();

        // 模拟长轮询：先 peek（无 answer），拿到 notify，然后 await。
        let (_, notify) = store.peek_answer(&sid).await.unwrap();
        let notify = notify.unwrap();

        // 在另一个任务中稍后提交 answer。
        let store2 = store.clone();
        let sid2 = sid.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            store2
                .submit_answer(
                    &sid2,
                    AnswerStatus::Accepted {
                        sdp: "answer".into(),
                    },
                )
                .await
                .ok();
        });

        // await notify（带超时保护）。
        let _ = tokio::time::timeout(Duration::from_secs(2), notify.notified()).await;

        // 再次 peek 应有 answer。
        let (answer, _) = store.peek_answer(&sid).await.unwrap();
        assert!(answer.is_some());
    }

    /// 每 device 并发会话上限（§4.6：10）。
    #[tokio::test]
    async fn max_sessions_per_device_enforced() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        for _ in 0..MAX_SESSIONS_PER_DEVICE {
            store
                .create_session(device_id, "sdp".into(), "cred".into())
                .await
                .unwrap();
        }
        let result = store
            .create_session(device_id, "sdp".into(), "cred".into())
            .await;
        assert!(matches!(result, Err(CreateSessionError::TooManySessions)));
    }

    /// TTL 过期后 peek 返回 NotFound。
    #[tokio::test]
    async fn expired_session_not_found() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "sdp".into(), "cred".into())
            .await
            .unwrap();
        // 手动把 created_at 置为过期（构造一个已过期的会话）。
        {
            let mut sessions = store.sessions.write().await;
            let session = sessions.get_mut(&sid).unwrap();
            session.created_at = Instant::now() - SIGNAL_SESSION_TTL - Duration::from_secs(1);
        }
        let result = store.peek_answer(&sid).await;
        assert!(matches!(result, Err(NotFound)));
    }

    /// cleanup_expired 移除过期会话。
    #[tokio::test]
    async fn cleanup_expired_removes_old_sessions() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "sdp".into(), "cred".into())
            .await
            .unwrap();
        // 置为过期。
        {
            let mut sessions = store.sessions.write().await;
            sessions.get_mut(&sid).unwrap().created_at =
                Instant::now() - SIGNAL_SESSION_TTL - Duration::from_secs(1);
        }
        let removed = store.cleanup_expired_sessions();
        assert_eq!(removed, 1);
        assert!(store.get_session(&sid).await.is_none());
    }

    /// offer 推送：桌面订阅后能收到。
    #[tokio::test]
    async fn push_offer_delivers_to_subscriber() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let _sub = store.subscribe_offers(device_id).await;

        let sid = store
            .create_session(device_id, "offer-sdp".into(), "cred".into())
            .await
            .unwrap();
        let session = store.get_session(&sid).await.unwrap();
        let pushed = store.push_offer(&session).await;
        assert!(pushed);
    }

    /// offer 推送：桌面未订阅时返回 false（§7.1 SSE 未就绪）。
    #[tokio::test]
    async fn push_offer_returns_false_when_not_subscribed() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sid = store
            .create_session(device_id, "offer-sdp".into(), "cred".into())
            .await
            .unwrap();
        let session = store.get_session(&sid).await.unwrap();
        let pushed = store.push_offer(&session).await;
        assert!(!pushed);
    }

    /// §7：正确 sub_id 的 unsubscribe 删除订阅。
    #[tokio::test]
    async fn unsubscribe_with_correct_sub_id_removes_subscription() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let sub = store.subscribe_offers(device_id).await;
        assert!(store.has_subscriber(device_id).await);
        store.unsubscribe_offers(device_id, sub.sub_id).await;
        assert!(!store.has_subscriber(device_id).await);
    }

    /// §7 根因修复：老任务用过期 sub_id unsubscribe 不删新订阅。
    /// 时序：A 订阅 → B 重订阅（替换，新 sub_id）→ A 退出用过期 sub_id unsubscribe
    /// → 不应删除 B 的订阅。
    #[tokio::test]
    async fn unsubscribe_with_stale_sub_id_keeps_new_subscription() {
        let store = make_store();
        let device_id = Uuid::new_v4();

        let sub_a = store.subscribe_offers(device_id).await;
        let sub_b = store.subscribe_offers(device_id).await;
        // B 替换 A，sub_id 单调递增。
        assert_ne!(sub_a.sub_id, sub_b.sub_id);
        assert!(store.has_subscriber(device_id).await);

        // A 的任务退出，用过期 sub_id unsubscribe——不应删 B。
        store.unsubscribe_offers(device_id, sub_a.sub_id).await;
        assert!(
            store.has_subscriber(device_id).await,
            "stale sub_id must not evict the newer subscription"
        );

        // B 的 unsubscribe 才删除。
        store.unsubscribe_offers(device_id, sub_b.sub_id).await;
        assert!(!store.has_subscriber(device_id).await);
    }

    /// §7：sub_id 全局单调递增。
    #[tokio::test]
    async fn sub_ids_are_globally_unique_and_increasing() {
        let store = make_store();
        let d1 = Uuid::new_v4();
        let d2 = Uuid::new_v4();
        let s1 = store.subscribe_offers(d1).await;
        let s2 = store.subscribe_offers(d2).await;
        assert!(
            s2.sub_id > s1.sub_id,
            "sub_id must be monotonically increasing"
        );
    }

    /// §7：重订阅后 push_offer 命中新 sender（老 sender 被替换，不再消费）。
    #[tokio::test]
    async fn resubscribe_replaces_sender_so_push_hits_new_subscriber() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let _old = store.subscribe_offers(device_id).await;
        let _new = store.subscribe_offers(device_id).await;

        let sid = store
            .create_session(device_id, "offer-sdp".into(), "cred".into())
            .await
            .unwrap();
        let session = store.get_session(&sid).await.unwrap();
        // push_offer 仍返回 true（有订阅者）；该 offer 入新 sender 队列。
        assert!(store.push_offer(&session).await);
    }

    /// credential 透传不解析（服务器不解析 code/token）。
    #[tokio::test]
    async fn credential_passed_through_verbatim() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        // code 形式
        let sid = store
            .create_session(device_id, "sdp".into(), "123456".into())
            .await
            .unwrap();
        let session = store.get_session(&sid).await.unwrap();
        assert_eq!(session.credential, "123456");

        // token 形式
        let sid2 = store
            .create_session(device_id, "sdp".into(), "dvct_abc123".into())
            .await
            .unwrap();
        let session2 = store.get_session(&sid2).await.unwrap();
        assert_eq!(session2.credential, "dvct_abc123");
    }

    // ── SSE 订阅票据 ──────────────────────────────────────────────────────

    /// UT-ticket-01：签发后可兑换，且只能兑换一次。
    #[tokio::test]
    async fn ticket_redeemable_exactly_once() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let ticket = store.issue_ticket(device_id).await;

        assert!(store.redeem_ticket(&ticket, device_id).await);
        // 第二次兑换（已消费）失败。
        assert!(!store.redeem_ticket(&ticket, device_id).await);
    }

    /// UT-ticket-02：票据与设备绑定。
    #[tokio::test]
    async fn ticket_bound_to_device() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let ticket = store.issue_ticket(device_id).await;

        assert!(!store.redeem_ticket(&ticket, Uuid::new_v4()).await);
        // 错误设备的兑换也消费票据（防重放枚举）。
        assert!(!store.redeem_ticket(&ticket, device_id).await);
    }

    /// UT-ticket-03：未知票据拒绝。
    #[tokio::test]
    async fn unknown_ticket_rejected() {
        let store = make_store();
        assert!(!store.redeem_ticket("no-such-ticket", Uuid::new_v4()).await);
    }

    /// UT-ticket-04：过期票据拒绝（手工置过期，同 session TTL 测试手法）。
    #[tokio::test]
    async fn expired_ticket_rejected() {
        let store = make_store();
        let device_id = Uuid::new_v4();
        let ticket = store.issue_ticket(device_id).await;

        {
            let mut tickets = store.tickets.write().await;
            tickets.get_mut(&ticket).unwrap().expires_at = Instant::now() - Duration::from_secs(1);
        }
        assert!(!store.redeem_ticket(&ticket, device_id).await);
    }
}

//! 连接管理 + 凭据存储（webrtc-scan-direct-design §5）。
//!
//! 替代旧的 `server/` axum 模块。WebRTC DataChannel 建立后，webview JS 通过
//! invoke 调用 `inject_text`，文本进注入队列由 Enigo 顺序注入。
//!
//! 凭据（§3）：
//! - Pairing Code：6 位数字，桌面生成 + 桌面验证（§3.2），5min TTL 内可复用。
//! - Connection Token：`dvct_` + 43 字符 base64url，桌面签发，持久化，上限 100 FIFO。
//!
//! `AppState` 持有 `ConnectionState`（活跃时的连接管理器 + 凭据 + heartbeat 句柄）。

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::{Mutex as AsyncMutex, RwLock};
use tracing::{info, warn};

use crate::config::DropVoiceConfig;
use crate::error::{AppError, AppResult};
use crate::server::auth;
use crate::telemetry::BusinessMetrics;
use crate::text::{EnigoInjector, Injector};

/// 配对码 + 其签发时间（用于 5min TTL 校验）。
/// §6：pub 以便 `build_qr_payload`（命令路径与定时器路径共用）按引用读取当前码。
pub type PairingCode = Option<(String, DateTime<Utc>)>;

/// 连接令牌 FIFO 上限（§3.3，永不过期，上限 100 FIFO）。
const MAX_CONNECTION_TOKENS: usize = 100;

/// 一个已连接的手机客户端（经 WebRTC DataChannel）。
#[derive(Debug, Clone, Serialize)]
pub struct ConnectedClient {
    pub id: String,
    pub connected_at: DateTime<Utc>,
}

/// 一个排队中的文本注入任务。
#[derive(Debug)]
pub struct InjectionTask {
    pub client_id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

/// 连接管理器：跟踪已连接手机 + 顺序注入队列。
///
/// 多台手机同时发送时，注入队列顺序处理，避免光标处文本交错。
pub struct ConnectionManager {
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
    injection_queue: Arc<Mutex<VecDeque<InjectionTask>>>,
    is_processing: Arc<AtomicBool>,
    max_connections: usize,
    max_queue_size: usize,
    injector: Arc<dyn Injector>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::with_limits(usize::MAX, usize::MAX, Arc::new(EnigoInjector::new()))
    }

    pub fn with_limits(
        max_connections: usize,
        max_queue_size: usize,
        injector: Arc<dyn Injector>,
    ) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            injection_queue: Arc::new(Mutex::new(VecDeque::new())),
            is_processing: Arc::new(AtomicBool::new(false)),
            max_connections,
            max_queue_size,
            injector,
        }
    }

    /// 注册一个新连接的手机（DataChannel open 后调用）。
    pub async fn register(&self, id: String) -> AppResult<()> {
        let mut clients = self.clients.write().await;
        if clients.len() >= self.max_connections && !clients.contains_key(&id) {
            return Err(AppError::MaxDevicesReached {
                max: self.max_connections,
            });
        }
        clients.insert(
            id.clone(),
            ConnectedClient {
                id,
                connected_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// 注销一个连接（DataChannel close 后调用）。
    pub async fn unregister(&self, id: &str) {
        self.clients.write().await.remove(id);
    }

    pub async fn get_all(&self) -> Vec<ConnectedClient> {
        self.clients.read().await.values().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// 入队一个文本注入任务。
    pub fn enqueue_injection(&self, client_id: String, text: String) -> AppResult<()> {
        let task = InjectionTask {
            client_id,
            text,
            timestamp: Utc::now(),
        };
        let mut queue = self
            .injection_queue
            .lock()
            .map_err(|e| AppError::Internal(format!("queue lock poisoned: {e}")))?;
        if queue.len() >= self.max_queue_size {
            return Err(AppError::QueueFull {
                depth: queue.len(),
                max: self.max_queue_size,
            });
        }
        queue.push_back(task);
        Ok(())
    }

    pub fn queue_depth(&self) -> usize {
        self.injection_queue.lock().map(|q| q.len()).unwrap_or(0)
    }

    pub fn is_processing(&self) -> bool {
        self.is_processing.load(Ordering::Relaxed)
    }

    /// 启动后台注入队列处理任务（服务运行期间常驻）。
    pub fn start_processing_task(self: Arc<Self>, delay_ms: u64, metrics: Arc<BusinessMetrics>) {
        tokio::spawn(async move {
            info!("injection queue processor started");
            loop {
                let task = {
                    let mut queue = self.injection_queue.lock().unwrap();
                    queue.pop_front()
                };

                if let Some(task) = task {
                    self.is_processing.store(true, Ordering::Relaxed);
                    let chars = task.text.chars().count() as u64;
                    let start = std::time::Instant::now();

                    let result: AppResult<()> = self.injector.inject(&task.text, delay_ms);
                    let elapsed_ms = start.elapsed().as_millis() as f64;

                    match &result {
                        Ok(()) => {
                            metrics.record_injection(chars, elapsed_ms, true);
                            info!(
                                client_id = %task.client_id,
                                chars = chars,
                                elapsed_ms = %elapsed_ms,
                                "injection completed"
                            );
                        }
                        Err(e) => {
                            metrics.record_injection(chars, elapsed_ms, false);
                            warn!(
                                client_id = %task.client_id,
                                error = %e,
                                "injection failed"
                            );
                        }
                    }

                    self.is_processing.store(false, Ordering::Relaxed);
                } else {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        });
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 活跃时的连接状态（start_server 创建，stop_server 清除）。
pub struct ConnectionState {
    pub connection_manager: Arc<ConnectionManager>,
    /// 当前配对码 + 签发时间（§3.2）。
    pub pairing_code: Arc<RwLock<PairingCode>>,
    /// 已签发的连接令牌（内存 + 持久化，§3.3）。
    pub tokens: Arc<RwLock<Vec<String>>>,
    pub config: Arc<RwLock<DropVoiceConfig>>,
    pub metrics: Arc<BusinessMetrics>,
    pub start_time: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl ConnectionState {
    pub fn new(config: DropVoiceConfig) -> Self {
        let max_connections = config.server.max_connections;
        let queue_size = config.injection.queue_size;
        let metrics = Arc::new(BusinessMetrics::new());
        let connection_manager = Arc::new(ConnectionManager::with_limits(
            max_connections,
            queue_size,
            Arc::new(EnigoInjector::new()),
        ));
        // 加载持久化的连接令牌。
        let tokens = config.device.connected_tokens.clone().unwrap_or_default();
        Self {
            connection_manager,
            pairing_code: Arc::new(RwLock::new(None)),
            tokens: Arc::new(RwLock::new(tokens)),
            config: Arc::new(RwLock::new(config)),
            metrics,
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn mark_started(&self) {
        *self.start_time.write().await = Some(Utc::now());
    }

    pub async fn mark_stopped(&self) {
        *self.start_time.write().await = None;
    }

    /// 签发新的 6 位配对码并记录时间戳（§3.2）。
    pub async fn issue_pairing_code(&self) -> String {
        let code = auth::generate_pairing_code();
        *self.pairing_code.write().await = Some((code.clone(), Utc::now()));
        code
    }

    /// 读取当前配对码（纯读，不轮换，§6）。无码时返回 None。
    pub async fn current_pairing_code(&self) -> Option<String> {
        self.pairing_code
            .read()
            .await
            .as_ref()
            .map(|(c, _)| c.clone())
    }

    /// 若当前配对码已过期则重新签发（§3.2 / 人工验收场景 2：过期后 QR 更新为新码）。
    ///
    /// 前端每 1s 轮询 get_connection_info → build_qr_payload，过期即轮换，
    /// 保证二维码展示的永远是有效配对码。返回当前（或新签发）的 code。
    pub async fn rotate_pairing_code_if_expired(&self) -> String {
        let expiry = chrono::Duration::minutes(
            self.config
                .read()
                .await
                .security
                .pairing_code_expiry_minutes as i64,
        );
        let expired = {
            let guard = self.pairing_code.read().await;
            match guard.as_ref() {
                Some((_, issued_at)) => Utc::now() - *issued_at >= expiry,
                None => true,
            }
        };
        if expired {
            let code = self.issue_pairing_code().await;
            tracing::info!(pairing_code = %code, "pairing code rotated (previous expired)");
            code
        } else {
            let guard = self.pairing_code.read().await;
            guard.as_ref().map(|(c, _)| c.clone()).unwrap_or_default()
        }
    }

    /// 校验 credential（code 或 token），桌面本地比对（§3.2/§3.3）。
    ///
    /// 返回 `CredentialKind` 标识命中类型（首次配对命中 code，重连命中 token）。
    pub async fn validate_credential(&self, credential: &str) -> CredentialKind {
        // 1. 校验配对码（§3.2，5min TTL 内可复用）。
        let code_guard = self.pairing_code.read().await;
        if let Some((stored_code, issued_at)) = code_guard.as_ref() {
            let expiry_minutes = self
                .config
                .read()
                .await
                .security
                .pairing_code_expiry_minutes;
            if auth::verify_pairing_code(stored_code, *issued_at, credential, expiry_minutes) {
                return CredentialKind::Code;
            }
        }
        drop(code_guard);

        // 2. 校验连接令牌（§3.3，内存比对）。
        let tokens = self.tokens.read().await;
        if auth::verify_connection_token(credential, &tokens) {
            return CredentialKind::Token;
        }

        CredentialKind::Invalid
    }

    /// 添加连接令牌并持久化（§3.3，上限 100 FIFO）。
    pub async fn add_connection_token(&self, token: String) {
        {
            let mut tokens = self.tokens.write().await;
            tokens.push(token.clone());
            if tokens.len() > MAX_CONNECTION_TOKENS {
                let excess = tokens.len() - MAX_CONNECTION_TOKENS;
                tokens.drain(0..excess);
            }
            // 持久化到 config。
            let mut cfg = self.config.write().await;
            cfg.device.connected_tokens = Some(tokens.clone());
            if let Err(e) = cfg.save() {
                warn!(error = %e, "failed to persist connection tokens");
            }
        }
    }

    /// 签发新的连接令牌（首次配对成功后调用）。
    pub async fn issue_connection_token(&self) -> String {
        let token = auth::generate_connection_token();
        self.add_connection_token(token.clone()).await;
        token
    }
}

/// credential 校验结果（§5.1，webview JS 收到 offer 后 invoke validate_credential）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    /// 配对码命中（首次配对）。
    Code,
    /// 连接令牌命中（重连）。
    Token,
    /// 无效。
    Invalid,
}

/// 心跳句柄（stop 时优雅关闭）。
pub struct HeartbeatHandle {
    stop_tx: tokio::sync::watch::Sender<bool>,
    join: tokio::task::JoinHandle<()>,
}

impl HeartbeatHandle {
    pub async fn stop(self) {
        let _ = self.stop_tx.send(true);
        let _ = self.join.await;
    }
}

#[allow(dead_code)]
fn _unused_async_mutex() -> AsyncMutex<()> {
    AsyncMutex::new(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::MockInjector;

    fn with_mock(max_connections: usize, max_queue_size: usize) -> ConnectionManager {
        ConnectionManager::with_limits(
            max_connections,
            max_queue_size,
            Arc::new(MockInjector::new()),
        )
    }

    #[tokio::test]
    async fn register_and_unregister_client() {
        let cm = with_mock(usize::MAX, usize::MAX);
        cm.register("dev1".into()).await.unwrap();
        assert_eq!(cm.count().await, 1);
        cm.unregister("dev1").await;
        assert_eq!(cm.count().await, 0);
    }

    #[tokio::test]
    async fn register_enforces_max_connections() {
        let cm = with_mock(2, usize::MAX);
        cm.register("a".into()).await.unwrap();
        cm.register("b".into()).await.unwrap();
        let err = cm.register("c".into()).await.unwrap_err();
        assert_eq!(err.error_code(), "MAX_DEVICES_REACHED");
    }

    #[tokio::test]
    async fn enqueue_enforces_max_queue_size() {
        let cm = with_mock(usize::MAX, 2);
        cm.enqueue_injection("dev1".into(), "a".into()).unwrap();
        cm.enqueue_injection("dev1".into(), "b".into()).unwrap();
        let err = cm.enqueue_injection("dev1".into(), "c".into()).unwrap_err();
        assert_eq!(err.error_code(), "QUEUE_FULL");
    }

    fn test_state() -> ConnectionState {
        let mut config = DropVoiceConfig::default();
        config.server.max_connections = 5;
        config.injection.queue_size = 100;
        ConnectionState::new(config)
    }

    #[tokio::test]
    async fn issue_and_validate_pairing_code() {
        let state = test_state();
        let code = state.issue_pairing_code().await;
        assert_eq!(state.validate_credential(&code).await, CredentialKind::Code);
    }

    /// §6：current_pairing_code 是纯读（签发前 None，签发后 Some）。
    #[tokio::test]
    async fn current_pairing_code_pure_read() {
        let state = test_state();
        assert_eq!(state.current_pairing_code().await, None);
        let code = state.issue_pairing_code().await;
        assert_eq!(state.current_pairing_code().await, Some(code));
    }

    #[tokio::test]
    async fn validate_invalid_credential() {
        let state = test_state();
        state.issue_pairing_code().await;
        assert_eq!(
            state.validate_credential("wrong").await,
            CredentialKind::Invalid
        );
    }

    #[tokio::test]
    async fn issue_and_validate_connection_token() {
        let state = test_state();
        let token = state.issue_connection_token().await;
        assert!(token.starts_with("dvct_"));
        assert_eq!(
            state.validate_credential(&token).await,
            CredentialKind::Token
        );
    }

    #[tokio::test]
    async fn connection_tokens_fifo_cap_100() {
        let state = test_state();
        for _ in 0..150 {
            state
                .add_connection_token(auth::generate_connection_token())
                .await;
        }
        let tokens = state.tokens.read().await;
        assert_eq!(tokens.len(), MAX_CONNECTION_TOKENS);
    }

    #[tokio::test]
    async fn expired_pairing_code_rejected() {
        let state = test_state();
        // 签发 code，手动把时间戳置为过期。
        let code = state.issue_pairing_code().await;
        {
            let mut pc = state.pairing_code.write().await;
            if let Some((_, ref mut t)) = pc.as_mut() {
                *t = Utc::now() - chrono::Duration::minutes(10);
            }
        }
        assert_eq!(
            state.validate_credential(&code).await,
            CredentialKind::Invalid
        );
    }
}

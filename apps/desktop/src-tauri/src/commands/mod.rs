pub mod server;
pub mod settings;
pub mod window;

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::config::DropVoiceConfig;
use crate::connection::ConnectionState;
use crate::network::heartbeat::HeartbeatHandle;

/// `pairing_code_rotated` 事件载荷（§0.3/§6）：配对码到点轮换后 push 给前端。
#[derive(Clone, Serialize)]
pub struct PairingCodePayload {
    pub code: String,
    pub qr_payload: String,
}

/// `client_registered` 事件载荷（§0.3/§6）：DataChannel open、register_client 成功后 push。
#[derive(Clone, Serialize)]
pub struct ClientEvent {
    pub client_id: String,
}

/// start_server / get_connection_info 返回的连接信息。
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub running: bool,
    /// QR 载荷（dropvoice://pair?code=&device=&name=，§6.5）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_payload: Option<String>,
    pub active_connections: usize,
    /// 当前 6 位配对码（§3.2）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairing_code: Option<String>,
    /// 已连接客户端详情。
    #[serde(default)]
    pub clients: Vec<ClientInfo>,
    /// 当前注入队列深度。
    #[serde(default)]
    pub queue_depth: usize,
}

/// 已连接客户端信息（前端展示）。
#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub id: String,
    pub connected_at: String,
}

/// get_settings 返回的设置。
#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub delay_ms: u64,
    pub port: u16,
    pub max_text_length: usize,
    pub minimize_to_tray: bool,
}

/// Tauri 命令间共享的可变运行时状态。
pub struct AppState {
    /// 已加载的配置，同时持久化到 dropvoice.toml。
    pub config: Arc<RwLock<DropVoiceConfig>>,
    /// 资源目录（保留兼容，WebRTC 主路径不再需要静态文件）。
    #[allow(dead_code)]
    pub resource_dir: PathBuf,
    /// 活跃时的连接状态（start_server 创建，stop_server 清除）。
    pub connection_state: Arc<Mutex<Option<ConnectionState>>>,
    /// Heartbeat 句柄（活跃时存在）。
    pub heartbeat: Arc<Mutex<Option<HeartbeatHandle>>>,
    /// §6 配对码轮换定时器句柄（start_server 创建，stop_server abort）。
    pub rotation_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl AppState {
    pub fn new(config: DropVoiceConfig, resource_dir: PathBuf) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            resource_dir,
            connection_state: Arc::new(Mutex::new(None)),
            heartbeat: Arc::new(Mutex::new(None)),
            rotation_task: Arc::new(Mutex::new(None)),
        }
    }
}

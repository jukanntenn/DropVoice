//! 网络相关模块（webrtc-scan-direct-design §5，信令面已迁入 Rust）。
//!
//! 常驻网络子系统全部在 Rust 进程：heartbeat（设备注册/心跳 + pairing_token
//! 分发）、signaling（SSE 订阅监督 + offer 分发）、pairing_client（HTTP 客户端）。
//! webview 不再参与信令链路——其 JS 定时器/网络随窗口可见性被 WebView2 冻结，
//! 是"休眠唤醒后无法恢复连接"的根因。

pub mod heartbeat;
pub mod pairing_client;
pub mod signaling;

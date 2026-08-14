//! 网络相关模块（webrtc-scan-direct-design §5）。
//!
//! WebRTC 信令客户端运行在 webview JS（§5.1），Rust 侧只保留 heartbeat
//! （设备注册/心跳，§5.1 职责边界）+ pairing_client（HTTP 客户端）。

pub mod heartbeat;
pub mod pairing_client;

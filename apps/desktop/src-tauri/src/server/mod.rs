//! 凭据生成与校验（webrtc-scan-direct-design §3）。
//!
//! 旧 axum HTTP/WS 服务已删除（M3），仅保留 auth 模块（配对码 + 连接令牌生成/校验）。
//! 新的数据路径：手机 →DataChannel→ webview JS → invoke('inject_text') → ConnectionManager → Enigo。

pub mod auth;

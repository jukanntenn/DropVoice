use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use hyper::{Request, Response};
use hyper::header::{UPGRADE, SEC_WEBSOCKET_KEY};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use sha1::Digest;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use crate::text::injector;
use tracing::{info, error, warn};

/// Connection manager to track active connections
#[derive(Clone)]
pub struct ConnectionManager {
    connection_count: Arc<AtomicUsize>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connection_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn increment(&self) -> usize {
        self.connection_count.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn decrement(&self) -> usize {
        self.connection_count.fetch_sub(1, Ordering::SeqCst) - 1
    }

    pub fn get_count(&self) -> usize {
        self.connection_count.load(Ordering::SeqCst)
    }
}

/// Handle WebSocket upgrade and connection
pub async fn handle_websocket(
    req: Request<hyper::body::Incoming>,
    connection_manager: ConnectionManager,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Check for WebSocket upgrade request
    let headers = req.headers();
    if !headers.contains_key(UPGRADE) {
        return Ok(Response::new(Full::new(Bytes::new())));
    }

    // Upgrade the connection
    let upgrade_header = headers
        .get(UPGRADE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if upgrade_header.to_lowercase() != "websocket" {
        return Ok(Response::new(Full::new(Bytes::new())));
    }

    // Generate Sec-WebSocket-Accept header
    let ws_key = headers
        .get(SEC_WEBSOCKET_KEY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // WebSocket magic UUID
    const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    // Compute accept key: SHA-1(ws_key + WS_GUID) then base64 encode
    let mut hasher = sha1::Sha1::new();
    hasher.update(ws_key.as_bytes());
    hasher.update(WS_GUID.as_bytes());
    let accept_key = STANDARD.encode(hasher.finalize());

    // Spawn a task to handle the upgraded connection
    tokio::task::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                info!("WebSocket connection established");
                let conn_id = connection_manager.increment();
                info!("Active connections: {}", conn_id);

                handle_connection(upgraded, connection_manager).await;
            }
            Err(e) => {
                error!("Failed to upgrade connection: {}", e);
            }
        }
    });

    // Return 101 Switching Protocols response with proper WebSocket headers
    Ok(Response::builder()
        .status(101)
        .header(hyper::header::UPGRADE, "websocket")
        .header(hyper::header::CONNECTION, "Upgrade")
        .header("sec-websocket-accept", accept_key)
        .body(Full::new(Bytes::new()))
        .unwrap())
}

/// Handle an active WebSocket connection
async fn handle_connection(
    upgraded: Upgraded,
    connection_manager: ConnectionManager,
) {
    // Wrap upgraded with TokioIo to provide AsyncRead/AsyncWrite
    let io = TokioIo::new(upgraded);
    let ws_stream = WebSocketStream::from_raw_socket(io, tokio_tungstenite::tungstenite::protocol::Role::Server, None).await;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send initial connection confirmation
    if let Err(e) = ws_sender
        .send(Message::Text(
            serde_json::json!({"type": "connected", "message": "Connected to DropVoice"}).to_string()
        ))
        .await
    {
        error!("Failed to send connection confirmation: {}", e);
        let _ = connection_manager.decrement();
        return;
    }

    // Handle incoming messages
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    let text = msg.to_text().unwrap_or("");
                    info!("Received message: {}", text);

                    // Parse JSON message
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        if json["type"] == "text" {
                            if let Some(content) = json["content"].as_str() {
                                // Validate text length
                                if content.len() > 10000 {
                                    warn!("Text too long: {} characters", content.len());
                                    let _ = ws_sender
                                        .send(Message::Text(
                                            serde_json::json!({
                                                "type": "error",
                                                "message": "Text too long (max 10000 characters)"
                                            }).to_string()
                                        ))
                                        .await;
                                    continue;
                                }

                                // Inject text
                                match injector::inject_text(content) {
                                    Ok(_) => {
                                        info!("Text injected successfully");
                                        let _ = ws_sender
                                            .send(Message::Text(
                                                serde_json::json!({"type": "confirm", "message": "Text sent successfully"}).to_string()
                                            ))
                                            .await;
                                    }
                                    Err(e) => {
                                        error!("Failed to inject text: {}", e);
                                        let _ = ws_sender
                                            .send(Message::Text(
                                                serde_json::json!({"type": "error", "message": format!("Failed to inject text: {}", e)}).to_string()
                                            ))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    let conn_count = connection_manager.decrement();
    info!("WebSocket connection closed. Active connections: {}", conn_count);
}

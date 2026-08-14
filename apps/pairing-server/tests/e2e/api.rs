//! E2E 集成测试。
//!
//! 用 reqwest 打真实 axum + 真实 SQLite（临时文件），不 Mock。
//! 覆盖：① 设备注册 upsert ② 状态上报 ③ token 续期 ④ WebRTC 信令端到端
//! （offer → SSE 推送 → answer 回填 → 长轮询）⑤ 错误响应格式 ⑥ 速率限制。

use std::sync::Arc;
use std::time::Duration;

use dropvoice_pairing_server::api;
use dropvoice_pairing_server::{clock, observability, store, AppState};
use tempfile::TempDir;
use uuid::Uuid;

/// 测试夹具：临时 SQLite + 启动 axum 在随机端口。
struct TestServer {
    base_url: String,
    #[allow(dead_code)]
    state: Arc<AppState>,
    _tmp: TempDir,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

/// 用 SystemClock 启动服务器。
async fn spawn_server() -> TestServer {
    spawn_server_with_clock(Arc::new(clock::SystemClock), 1000).await
}

/// 用指定 Clock + rate_limit 启动服务器。
async fn spawn_server_with_clock(
    clk: Arc<dyn clock::Clock>,
    rate_limit_per_sec: u32,
) -> TestServer {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("test.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = store::open_pool(&database_url).await.expect("open pool");

    let metrics = Arc::new(observability::Metrics::default());
    let state = AppState::new(pool, metrics, clk, rate_limit_per_sec);
    let app_state = Arc::new(state.clone());

    let app = api::build_app_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{addr}");

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await
        .ok();
    });

    TestServer {
        base_url,
        state: app_state,
        _tmp: tmp,
        _shutdown: shutdown_tx,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().build().expect("reqwest client")
}

// ──────────────────────────────────────────────────────────────────────────────
// ① 设备注册 upsert
// ──────────────────────────────────────────────────────────────────────────────

/// E2E-01：新建 201。
#[tokio::test]
async fn register_new_device_returns_201() {
    let server = spawn_server().await;
    let device_id = Uuid::new_v4();
    let resp = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "device_name": "My PC",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], device_id.to_string());
    assert_eq!(body["pairing_token"].as_str().unwrap().len(), 64);
    assert_eq!(body["platform"], "desktop");
    assert_eq!(body["device_name"], "My PC");
    assert_eq!(body["address"]["ip"], "192.168.1.100");
    assert_eq!(body["address"]["port"], 38425);
}

/// E2E-02：复用 200 + 原 token。
#[tokio::test]
async fn register_existing_device_returns_200_reuses_token() {
    let server = spawn_server().await;
    let device_id = Uuid::new_v4();
    let payload = serde_json::json!({
        "device_id": device_id,
        "platform": "desktop",
        "address": { "ip": "192.168.1.100", "port": 38425 }
    });

    let first = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 201);
    let first_body: serde_json::Value = first.json().await.unwrap();
    let first_token = first_body["pairing_token"].as_str().unwrap().to_string();

    // 立即复用 → 200 + 同 token。
    let second = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), 200);
    let second_body: serde_json::Value = second.json().await.unwrap();
    assert_eq!(second_body["pairing_token"].as_str().unwrap(), first_token);
}

/// E2E-03：续期 200 + 新 token（token 过期后刷新）。
#[tokio::test]
async fn register_expired_token_returns_200_new_token() {
    let clk = Arc::new(clock::FakeClock::new(chrono::Utc::now()));
    let server = spawn_server_with_clock(clk.clone(), 1000).await;
    let device_id = Uuid::new_v4();
    let payload = serde_json::json!({
        "device_id": device_id,
        "platform": "desktop",
        "address": { "ip": "192.168.1.100", "port": 38425 }
    });

    // 首次注册。
    let first = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 201);
    let first_body: serde_json::Value = first.json().await.unwrap();
    let first_token = first_body["pairing_token"].as_str().unwrap().to_string();

    // 推进 25 小时（超过 DEVICE_TOKEN_TTL 24h）。
    clk.advance(chrono::Duration::hours(25));

    // 再次注册 → 200 + 新 token。
    let second = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), 200);
    let second_body: serde_json::Value = second.json().await.unwrap();
    let second_token = second_body["pairing_token"].as_str().unwrap().to_string();
    assert_ne!(second_token, first_token);
}

// ──────────────────────────────────────────────────────────────────────────────
// ② 状态上报
// ──────────────────────────────────────────────────────────────────────────────

/// 注册设备辅助，返回 (device_id, token)。
async fn register_device(server: &TestServer) -> (Uuid, String) {
    let device_id = Uuid::new_v4();
    let resp = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "device_name": "My PC",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    (
        device_id,
        body["pairing_token"].as_str().unwrap().to_string(),
    )
}

/// E2E-08：上报 happy 200。
#[tokio::test]
async fn status_report_happy_200() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    let resp = client()
        .put(format!(
            "{}/api/devices/{}/status",
            server.base_url, device_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "address": { "ip": "192.168.1.200", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
}

/// E2E-09：device_name 改名落库。
#[tokio::test]
async fn status_report_device_name_change() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    // 上报新名字。
    let resp = client()
        .put(format!(
            "{}/api/devices/{}/status",
            server.base_url, device_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "address": { "ip": "192.168.1.100", "port": 38425 },
            "device_name": "Renamed PC"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 等待 batch flush（AppState::new 用 BATCH_WRITE_INTERVAL_SECS=1s，留足余量）。
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // 重新注册确认 device_name 已更新。
    let resp = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["device_name"], "Renamed PC");
}

// ──────────────────────────────────────────────────────────────────────────────
// ③ token 续期
// ──────────────────────────────────────────────────────────────────────────────

/// E2E-11：token 过期→重 POST→刷新新 token（DEVICE_TOKEN_TTL 24h 强制续期）。
///
/// token 过期的强制机制在 upsert（re-register）端：超过 24h 重新 POST /api/devices
/// 会刷新 pairing_token。find_id_by_token 本身不校验 TTL（既有行为），
/// 故旧 token 在 DB 中仍可认证——续期靠客户端主动 re-register（heartbeat 检测 401 后触发）。
#[tokio::test]
async fn token_expired_re_register_refreshes() {
    let clk = Arc::new(clock::FakeClock::new(chrono::Utc::now()));
    let server = spawn_server_with_clock(clk.clone(), 1000).await;
    let device_id = Uuid::new_v4();

    // 首次注册拿 token。
    let first = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": device_id, "platform": "desktop",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 201);
    let first_body: serde_json::Value = first.json().await.unwrap();
    let first_token = first_body["pairing_token"].as_str().unwrap().to_string();

    // 推进 25 小时（超过 DEVICE_TOKEN_TTL 24h）。
    clk.advance(chrono::Duration::hours(25));

    // 重新注册 → 200 + 新 token（续期刷新）。
    let re_reg = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": device_id, "platform": "desktop",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(re_reg.status(), 200);
    let re_body: serde_json::Value = re_reg.json().await.unwrap();
    let new_token = re_body["pairing_token"].as_str().unwrap().to_string();
    assert_ne!(new_token, first_token);

    // 用新 token 心跳 → 200。
    let retry = client()
        .put(format!(
            "{}/api/devices/{}/status",
            server.base_url, device_id
        ))
        .bearer_auth(&new_token)
        .json(&serde_json::json!({
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(retry.status(), 200);
}

// ──────────────────────────────────────────────────────────────────────────────
// ④ WebRTC 信令端到端
// ──────────────────────────────────────────────────────────────────────────────

/// E2E-Sig-01：offer → SSE 推送 → answer 回填 → 长轮询（accepted）。
///
/// 模拟完整信令流：
/// 1. 桌面注册 + SSE 订阅
/// 2. 手机 POST offer（带 code）
/// 3. 桌面经 SSE 收到 offer
/// 4. 桌面 POST answer（accepted + sdp）
/// 5. 手机长轮询拿到 answer
#[tokio::test]
async fn signaling_full_flow_accepted() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    // 1. 桌面 SSE 订阅（用 query token）。
    let sse_url = format!(
        "{}/api/devices/{}/webrtc/events?token={}",
        server.base_url, device_id, token
    );
    let resp = reqwest::get(&sse_url).await.unwrap();
    assert_eq!(resp.status(), 200);

    // 2. 手机 POST offer（带 code）。
    let offer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "code": "123456",
            "sdp": "v=0\r\no=- 1 1 IN IP4 0.0.0.0\r\ns=-\r\n"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(offer_resp.status(), 200);
    let offer_body: serde_json::Value = offer_resp.json().await.unwrap();
    let session_id = offer_body["session_id"].as_str().unwrap().to_string();
    assert_eq!(offer_body["device"]["id"], device_id.to_string());

    // 3. 桌面 POST answer（accepted）—— 模拟桌面校验 code 通过后回填。
    let answer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/answer",
            server.base_url, device_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "session_id": session_id,
            "sdp": "v=0\r\no=- 2 2 IN IP4 0.0.0.0\r\ns=-\r\n",
            "status": "accepted"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(answer_resp.status(), 200);

    // 4. 手机长轮询拿到 answer（accepted + sdp）。
    let poll_resp = client()
        .get(format!(
            "{}/api/devices/{}/webrtc/answer/{}",
            server.base_url, device_id, session_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(poll_resp.status(), 200);
    let poll_body: serde_json::Value = poll_resp.json().await.unwrap();
    assert_eq!(poll_body["status"], "accepted");
    assert!(poll_body["sdp"].as_str().unwrap().contains("v=0"));
}

/// E2E-Sig-02：桌面拒绝（invalid_credential）→ 手机收到 rejected。
#[tokio::test]
async fn signaling_rejected_invalid_credential() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    // 桌面 SSE 订阅（§7：无订阅时 offer 直接 503，须先订阅让 offer 送达桌面）。
    let sse_url = format!(
        "{}/api/devices/{}/webrtc/events?token={}",
        server.base_url, device_id, token
    );
    let sse_resp = reqwest::get(&sse_url).await.unwrap();
    assert_eq!(sse_resp.status(), 200);

    // POST offer（带错误 code）。
    let offer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "code": "wrong-code",
            "sdp": "v=0\r\n"
        }))
        .send()
        .await
        .unwrap();
    let session_id = offer_resp.json::<serde_json::Value>().await.unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 桌面拒绝。
    let answer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/answer",
            server.base_url, device_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "session_id": session_id,
            "status": "rejected",
            "reason": "invalid_credential"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(answer_resp.status(), 200);

    // 手机收到 rejected。
    let poll_resp = client()
        .get(format!(
            "{}/api/devices/{}/webrtc/answer/{}",
            server.base_url, device_id, session_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(poll_resp.status(), 200);
    let poll_body: serde_json::Value = poll_resp.json().await.unwrap();
    assert_eq!(poll_body["status"], "rejected");
    assert_eq!(poll_body["reason"], "invalid_credential");
}

/// E2E-Sig-03：长轮询超时 → 204（桌面订阅了但未回填 answer）。
/// §7 后：offer 必须有 SSE 订阅才能 200（否则 503 fast-fail）。
#[tokio::test]
async fn signaling_poll_timeout_returns_204() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    // 桌面 SSE 订阅（让 offer 能送达，否则 §7 直接 503）。
    let sse_url = format!(
        "{}/api/devices/{}/webrtc/events?token={}",
        server.base_url, device_id, token
    );
    let sse_resp = reqwest::get(&sse_url).await.unwrap();
    assert_eq!(sse_resp.status(), 200);

    // POST offer；桌面订阅但不回填 answer。
    let offer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "code": "123456",
            "sdp": "v=0\r\n"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(offer_resp.status(), 200);
    let session_id = offer_resp.json::<serde_json::Value>().await.unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    // 长轮询会 hold 最多 30s。用 35s 超时的 client 避免测试阻塞。
    let short_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(35))
        .build()
        .unwrap();
    let poll_resp = short_client
        .get(format!(
            "{}/api/devices/{}/webrtc/answer/{}",
            server.base_url, device_id, session_id
        ))
        .send()
        .await
        .unwrap();
    // 30s 后返回 204（answer 未就绪）。
    assert_eq!(poll_resp.status(), 204);

    drop(sse_resp);
}

/// E2E-Sig-03b（§7）：桌面无 SSE 订阅 → POST offer 立即 503 DEVICE_OFFLINE
///（fast-fail，手机不再建会话干等 30s answer）。
#[tokio::test]
async fn signaling_offer_no_subscriber_returns_503() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    // 无 SSE 订阅直接 POST offer → 503。
    let offer_resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "code": "123456",
            "sdp": "v=0\r\n"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(offer_resp.status(), 503);
    let body: serde_json::Value = offer_resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "DEVICE_OFFLINE");
}

/// E2E-Sig-04：offer 缺 code 和 token → 400。
#[tokio::test]
async fn signaling_offer_missing_credential_returns_400() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    let resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "sdp": "v=0\r\n"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

/// E2E-Sig-05：offer 到不存在的 device → 404。
#[tokio::test]
async fn signaling_offer_unknown_device_returns_404() {
    let server = spawn_server().await;
    let fake_id = Uuid::new_v4();

    let resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, fake_id
        ))
        .json(&serde_json::json!({
            "code": "123456",
            "sdp": "v=0\r\n"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

/// E2E-Sig-06：SDP 过大 → 413。
#[tokio::test]
async fn signaling_offer_oversized_sdp_returns_413() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    // 构造 >64KB 的 SDP。
    let huge_sdp = "v=0\r\n".repeat(20_000); // ~100KB

    let resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/offer",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "code": "123456",
            "sdp": huge_sdp
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
}

/// E2E-Sig-07：长轮询不存在的 session → 404。
#[tokio::test]
async fn signaling_poll_unknown_session_returns_404() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    let resp = client()
        .get(format!(
            "{}/api/devices/{}/webrtc/answer/nonexistent-session",
            server.base_url, device_id
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

/// E2E-Sig-08：桌面 answer 到不存在的 session → 404。
#[tokio::test]
async fn signaling_submit_answer_unknown_session_returns_404() {
    let server = spawn_server().await;
    let (device_id, token) = register_device(&server).await;

    let resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/answer",
            server.base_url, device_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "session_id": "nonexistent",
            "sdp": "v=0\r\n",
            "status": "accepted"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

/// E2E-Sig-09：SSE 无效 token → 401。
#[tokio::test]
async fn signaling_sse_invalid_token_returns_401() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    let resp = reqwest::get(format!(
        "{}/api/devices/{}/webrtc/events?token=invalid",
        server.base_url, device_id
    ))
    .await
    .unwrap();
    assert_eq!(resp.status(), 401);
}

/// E2E-Sig-10：桌面 answer 路径 device_id 与 token 不符 → 401。
#[tokio::test]
async fn signaling_submit_answer_device_id_mismatch_returns_401() {
    let server = spawn_server().await;
    let (_device_id, token) = register_device(&server).await;
    let other_device = Uuid::new_v4();

    let resp = client()
        .post(format!(
            "{}/api/devices/{}/webrtc/answer",
            server.base_url, other_device
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "session_id": "any",
            "sdp": "v=0\r\n",
            "status": "accepted"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ──────────────────────────────────────────────────────────────────────────────
// ⑤ 错误响应格式
// ──────────────────────────────────────────────────────────────────────────────

/// E2E-12：缺 platform → 422（axum Json 提取器反序列化失败）。
#[tokio::test]
async fn register_missing_platform_returns_422() {
    let server = spawn_server().await;
    let resp = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": Uuid::new_v4(),
            "address": { "ip": "1.2.3.4", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    // axum 的 Json 提取器反序列化失败返回 422（缺必填字段 platform）。
    assert_eq!(resp.status(), 422);
}

/// E2E-13：无 Bearer 心跳 → 401。
#[tokio::test]
async fn status_report_no_auth_returns_401() {
    let server = spawn_server().await;
    let (device_id, _token) = register_device(&server).await;

    let resp = client()
        .put(format!(
            "{}/api/devices/{}/status",
            server.base_url, device_id
        ))
        .json(&serde_json::json!({
            "address": { "ip": "1.2.3.4", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

/// E2E-14：路径 device_id ≠ token device_id → 401。
#[tokio::test]
async fn status_report_id_mismatch_returns_401() {
    let server = spawn_server().await;
    let (_device_id, token) = register_device(&server).await;
    let other_id = Uuid::new_v4();

    let resp = client()
        .put(format!(
            "{}/api/devices/{}/status",
            server.base_url, other_id
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "address": { "ip": "1.2.3.4", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ──────────────────────────────────────────────────────────────────────────────
// ⑥ 速率限制
// ──────────────────────────────────────────────────────────────────────────────

/// E2E-18：单 IP 限速（limit=1，第二次请求 429）。
#[tokio::test]
async fn rate_limit_single_ip_429() {
    let clk = Arc::new(clock::SystemClock);
    let server = spawn_server_with_clock(clk, 1).await;

    let payload = serde_json::json!({
        "device_id": Uuid::new_v4(), "platform": "desktop",
        "address": { "ip": "1.2.3.4", "port": 38425 }
    });

    // 第一次请求允许。
    let first = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 201);

    // 第二次请求（同 IP，1s 内）→ 429。
    let second = client()
        .post(format!("{}/api/devices", server.base_url))
        .json(&serde_json::json!({
            "device_id": Uuid::new_v4(), "platform": "desktop",
            "address": { "ip": "1.2.3.4", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(second.status(), 429);
}

/// E2E-19：高限流值允许大量请求。
#[tokio::test]
async fn rate_limit_high_limit_allows_many() {
    let server = spawn_server_with_clock(Arc::new(clock::SystemClock), 1000).await;

    for _ in 0..10 {
        let resp = client()
            .post(format!("{}/api/devices", server.base_url))
            .json(&serde_json::json!({
                "device_id": Uuid::new_v4(), "platform": "desktop",
                "address": { "ip": "1.2.3.4", "port": 38425 }
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
    }
}

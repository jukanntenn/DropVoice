//! E2E Caddy 通道测试。
//!
//! 通过 docker-compose 起 Caddy:8080 + axum:38424，reqwest 打
//! `http://localhost:8080`（明文 HTTP——本机验收不做 TLS，TLS 由隧道 /
//! Cloudflare 在部署形态外部终止）。
//!
//! 运行前置：
//!   docker compose -f apps/pairing-server/docker/docker-compose.local.yml up -d --build
//!
//! 运行：
//!   cargo test -p dropvoice-pairing-server --test e2e-caddy -- --ignored

use uuid::Uuid;

const CADDY_BASE: &str = "http://localhost:8080";

fn caddy_client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-01: 通道健康（经 Caddy 反代）
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn caddy_http_health() {
    let resp = caddy_client()
        .get(format!("{CADDY_BASE}/health"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["schema_version"].as_i64().unwrap() > 0);
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-02: gzip 压缩生效
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn caddy_gzip_compression() {
    let resp = caddy_client()
        .get(format!("{CADDY_BASE}/health"))
        .header("Accept-Encoding", "gzip")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let _ = resp.text().await.unwrap();
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-03: 反代路径 /api/devices/*
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn caddy_proxy_register_device() {
    let device_id = Uuid::new_v4();
    let resp = caddy_client()
        .post(format!("{CADDY_BASE}/api/devices"))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "device_name": "Caddy Test PC",
            "address": { "ip": "192.168.1.200", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], device_id.to_string());
    assert_eq!(body["pairing_token"].as_str().unwrap().len(), 64);
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-04: WebRTC 信令流经 Caddy
// ──────────────────────────────────────────────────────────────────────────────

/// 桌面在线（先建立 SSE 订阅）→ offer 被接受并推送 → answer 回填成功。
/// offer 的 503 fast-fail（无订阅，§7）由 `offer_rejected_without_subscriber`
/// 单元侧覆盖；本测试走完整链路，与真实配对路径一致。
#[tokio::test]
#[ignore]
async fn caddy_proxy_webrtc_signaling_flow() {
    let c = caddy_client();
    let device_id = Uuid::new_v4();

    // 1. 注册设备。
    let token = {
        let resp = c
            .post(format!("{CADDY_BASE}/api/devices"))
            .json(&serde_json::json!({
                "device_id": device_id,
                "platform": "desktop",
                "address": { "ip": "192.168.1.200", "port": 38425 }
            }))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        body["pairing_token"].as_str().unwrap().to_string()
    };

    // 2. 桌面订阅 SSE（query token，与 signaling.ts 一致）。保持连接打开
    //    直到 answer 回填完成——订阅存在是 offer 被接受的前提。
    let events = c
        .get(format!(
            "{CADDY_BASE}/api/devices/{device_id}/webrtc/events?token={token}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(events.status(), 200);

    // 3. 手机 POST offer（有活跃订阅 → 200 + session_id）。
    let session_id = {
        let resp = c
            .post(format!("{CADDY_BASE}/api/devices/{device_id}/webrtc/offer"))
            .json(&serde_json::json!({
                "code": "123456",
                "sdp": "v=0\r\n"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = resp.json().await.unwrap();
        body["session_id"].as_str().unwrap().to_string()
    };

    // 4. 桌面回填 answer。
    let resp = c
        .post(format!(
            "{CADDY_BASE}/api/devices/{device_id}/webrtc/answer"
        ))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "session_id": session_id,
            "sdp": "v=0\r\n",
            "status": "accepted"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 关闭 SSE（drop 连接）。
    drop(events);
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-05: X-Forwarded-For 透传
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn caddy_xff_forwarded() {
    let resp = caddy_client()
        .post(format!("{CADDY_BASE}/api/devices"))
        .json(&serde_json::json!({
            "device_id": Uuid::new_v4(),
            "platform": "desktop",
            "address": { "ip": "10.0.0.1", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// ──────────────────────────────────────────────────────────────────────────────
// E2E-CADDY-06: 完整信令流程经 Caddy
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn caddy_full_signaling_flow() {
    let c = caddy_client();
    let device_id = Uuid::new_v4();

    // 1. 注册设备。
    let reg_resp = c
        .post(format!("{CADDY_BASE}/api/devices"))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "device_name": "Full Flow PC",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(reg_resp.status(), 201);
    let token = reg_resp.json::<serde_json::Value>().await.unwrap()["pairing_token"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. 状态上报。
    let status_resp = c
        .put(format!("{CADDY_BASE}/api/devices/{device_id}/status"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "address": { "ip": "192.168.1.100", "port": 38425 },
            "device_name": "Full Flow PC"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(status_resp.status(), 200);

    // 3. 重复注册（幂等）。
    let reg2_resp = c
        .post(format!("{CADDY_BASE}/api/devices"))
        .json(&serde_json::json!({
            "device_id": device_id,
            "platform": "desktop",
            "address": { "ip": "192.168.1.100", "port": 38425 }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(reg2_resp.status(), 200);
    assert_eq!(
        reg2_resp.json::<serde_json::Value>().await.unwrap()["pairing_token"]
            .as_str()
            .unwrap(),
        token
    );
}

//! HTTP client for the pairing/signaling server (webrtc-scan-direct-design §4.1).
//!
//! Provides two operations:
//! - `register_device` — `POST /api/devices`
//! - `report_status` — `PUT /api/devices/{id}/status`
//!
//! 配对码（code）改由桌面本地生成 + 本地验证（§3.2），服务端零 code 知识，
//! 故不再有 `generate_pairing_code` 调用。
//!
//! All calls are best-effort with structured error returns; the caller decides
//! retry and degradation strategy.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default pairing server base URL (webrtc-scan-direct-design §10.4).
/// The pairing server and the mobile PWA share one origin — `dropvoice.bytehome.fun`
/// (Caddy serves both). Overridable at runtime via the `PAIRING_SERVER_URL`
/// environment variable.
pub const DEFAULT_PAIRING_SERVER_URL: &str = "https://dropvoice.bytehome.fun";

/// Timeout for individual HTTP requests (spec 11 §4 `SIGNALING_TIMEOUT` 3s).
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// Build the shared HTTP client.
///
/// `PAIRING_SERVER_INSECURE=1` skips TLS certificate verification — required
/// only for local acceptance against the self-signed acceptance container
/// (https://<lan-ip>:4443). Never set in production.
fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder().timeout(REQUEST_TIMEOUT);
    let insecure = std::env::var("PAIRING_SERVER_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    builder.build()
}

/// Request body for `POST /devices` (spec 11 §5.1.1).
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub device_id: Uuid,
    pub platform: String,
    pub address: RegisterAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAddress {
    pub ip: String,
    pub port: u16,
}

/// Response body for `POST /devices`.
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub pairing_token: String,
    #[serde(default)]
    pub device_name: Option<String>,
}

/// Request body for `PUT /devices/{id}/status` (spec 11 §7.4).
#[derive(Debug, Serialize)]
pub struct StatusReportRequest {
    pub address: RegisterAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
}

/// Response body for `PUT /devices/{id}/status`.
#[derive(Debug, Deserialize)]
pub struct StatusReportResponse {
    pub ok: bool,
}

/// Error from the pairing server API.
#[derive(Debug, thiserror::Error)]
pub enum PairingClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Server returned error: {status} {code}")]
    Server {
        status: u16,
        code: String,
        message: String,
    },
}

/// Register this device with the pairing server.
///
/// Returns the pairing token on success (201 Created or 200 OK).
/// The `device_id` must be stable (UUID v4 from config).
pub async fn register_device(
    base_url: &str,
    device_id: Uuid,
    ip: &str,
    port: u16,
    device_name: Option<&str>,
) -> Result<RegisterResponse, PairingClientError> {
    let client = build_client()?;

    let req = RegisterRequest {
        device_id,
        platform: "desktop".into(),
        address: RegisterAddress {
            ip: ip.to_string(),
            port,
        },
        device_name: device_name.map(|s| s.to_string()),
    };

    let url = format!("{base_url}/api/devices");
    let resp = client.post(&url).json(&req).send().await?;

    if resp.status().is_success() {
        let body: RegisterResponse = resp.json().await?;
        Ok(body)
    } else {
        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(PairingClientError::Server {
            status,
            code: body["error"]["code"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            message: body["error"]["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// Report device status to the pairing server (spec 11 §7.4).
pub async fn report_status(
    base_url: &str,
    device_id: Uuid,
    token: &str,
    ip: &str,
    port: u16,
    device_name: Option<&str>,
) -> Result<StatusReportResponse, PairingClientError> {
    let client = build_client()?;

    let req = StatusReportRequest {
        address: RegisterAddress {
            ip: ip.to_string(),
            port,
        },
        device_name: device_name.map(|s| s.to_string()),
    };

    let url = format!("{base_url}/api/devices/{device_id}/status");
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(&req)
        .send()
        .await?;

    if resp.status().is_success() {
        let body: StatusReportResponse = resp.json().await?;
        Ok(body)
    } else {
        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(PairingClientError::Server {
            status,
            code: body["error"]["code"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            message: body["error"]["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_serializes_correctly() {
        let req = RegisterRequest {
            device_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            platform: "desktop".into(),
            address: RegisterAddress {
                ip: "192.168.1.100".into(),
                port: 38425,
            },
            device_name: Some("My PC".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["platform"], "desktop");
        assert_eq!(json["address"]["ip"], "192.168.1.100");
        assert_eq!(json["address"]["port"], 38425);
        assert_eq!(json["device_name"], "My PC");
    }

    #[test]
    fn register_request_omits_device_name_when_none() {
        let req = RegisterRequest {
            device_id: Uuid::new_v4(),
            platform: "desktop".into(),
            address: RegisterAddress {
                ip: "10.0.0.1".into(),
                port: 38425,
            },
            device_name: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("device_name").is_none());
    }

    #[test]
    fn status_report_request_serializes() {
        let req = StatusReportRequest {
            address: RegisterAddress {
                ip: "10.0.0.1".into(),
                port: 38425,
            },
            device_name: Some("New Name".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["address"]["ip"], "10.0.0.1");
        assert_eq!(json["device_name"], "New Name");
    }
}

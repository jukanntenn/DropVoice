//! Device 领域模型（spec 11 §2.2）。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 设备局域网地址（B 线桌面端本地服务器地址）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeviceAddress {
    /// 设备局域网 IPv4/IPv6 地址。
    #[schema(example = "192.168.1.100")]
    pub ip: String,
    /// 桌面端本地服务器端口（默认 38425）。
    #[schema(example = 38425)]
    pub port: u16,
}

/// 设备平台。当前仅 desktop 会注册到配对服务器。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
#[schema(example = "desktop")]
pub enum Platform {
    Desktop,
}

/// 设备领域实体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 客户端生成的 UUID v4（spec 11 §14，永不变更）。
    pub id: Uuid,
    pub platform: Platform,
    /// 固化的设备名（spec 11 §6.6）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    pub address: DeviceAddress,
    /// 配对 token（64 字符）。续期时更新。
    pub pairing_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 设备状态（在线/最后在线时间）。与持久化层交互时单独读写。
#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub is_online: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Maximum length for device_name (OpenAPI maxLength:64, spec 11 §6.6).
const DEVICE_NAME_MAX_LEN: usize = 64;

/// `POST /devices` 请求体（与 `pairing-server.yaml` `DeviceRegisterRequest` 一致）。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DeviceRegisterRequest {
    /// 客户端生成的 UUID v4，持久化在 config.toml，永不变更。
    pub device_id: Uuid,
    /// 当前仅 desktop 会注册到配对服务器。
    pub platform: Platform,
    /// 固化的设备名（§6.6）。可选字段；缺省时由服务端记录为空，
    /// 手机端 UI 用 `name (ip)` 组合显示。
    #[serde(default)]
    #[schema(max_length = 64)]
    pub device_name: Option<String>,
    /// 桌面端当前局域网地址。
    pub address: DeviceAddress,
}

impl DeviceRegisterRequest {
    /// Pre-req-10: validate device_name length (max 64 chars).
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if let Some(ref name) = self.device_name {
            if name.len() > DEVICE_NAME_MAX_LEN {
                return Err(crate::error::AppError::InvalidRequest(format!(
                    "device_name exceeds maximum length of {DEVICE_NAME_MAX_LEN} characters"
                )));
            }
        }
        Ok(())
    }
}

/// `POST /devices` / 复用响应体（`DeviceRegisterResponse`）。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeviceRegisterResponse {
    /// 等同请求的 `device_id`。
    pub id: Uuid,
    pub platform: Platform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    pub address: DeviceAddress,
    /// 64 字符随机字符串，用于后续认证。续期时此字段更新。
    pub pairing_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<Device> for DeviceRegisterResponse {
    fn from(d: Device) -> Self {
        Self {
            id: d.id,
            platform: d.platform,
            device_name: d.device_name,
            address: d.address,
            pairing_token: d.pairing_token,
            created_at: d.created_at,
        }
    }
}

/// `PUT /devices/{id}/status` 请求体（`StatusReportRequest`）。
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StatusReportRequest {
    pub address: DeviceAddress,
    /// 用户改名后携带，刷新服务端记录。
    #[serde(default)]
    #[schema(max_length = 64)]
    pub device_name: Option<String>,
}

impl StatusReportRequest {
    /// Pre-req-10: validate device_name length (max 64 chars).
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if let Some(ref name) = self.device_name {
            if name.len() > DEVICE_NAME_MAX_LEN {
                return Err(crate::error::AppError::InvalidRequest(format!(
                    "device_name exceeds maximum length of {DEVICE_NAME_MAX_LEN} characters"
                )));
            }
        }
        Ok(())
    }
}

/// `PUT /devices/{id}/status` 响应体（`StatusReportResponse`）。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusReportResponse {
    #[schema(example = true)]
    pub ok: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UT-domain-01：DeviceAddress serde 往返。
    #[test]
    fn device_address_serde_roundtrip() {
        let addr = DeviceAddress {
            ip: "192.168.1.100".to_string(),
            port: 38425,
        };
        let json = serde_json::to_string(&addr).unwrap();
        let decoded: DeviceAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, decoded);
    }

    /// UT-domain-02：Platform::Desktop 序列化为 "desktop"。
    #[test]
    fn platform_desktop_serializes_lowercase() {
        let p = Platform::Desktop;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "\"desktop\"");
    }

    /// UT-domain-03：Platform 拒绝枚举外值。
    #[test]
    fn platform_rejects_unknown_value() {
        let result: Result<Platform, _> = serde_json::from_str("\"mobile\"");
        assert!(result.is_err());
    }

    /// UT-domain-04：device_name=None 不出现在 JSON。
    #[test]
    fn device_name_none_not_in_json() {
        let device = Device {
            id: uuid::Uuid::new_v4(),
            platform: Platform::Desktop,
            device_name: None,
            address: DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
            pairing_token: "token".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_value(&device).unwrap();
        assert!(json.get("device_name").is_none());
    }

    /// UT-domain-05：Device→DeviceRegisterResponse From。
    #[test]
    fn device_to_register_response_from() {
        let device = Device {
            id: uuid::Uuid::new_v4(),
            platform: Platform::Desktop,
            device_name: Some("My PC".into()),
            address: DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
            pairing_token: "token123".into(),
            created_at: chrono::Utc::now(),
        };
        let resp: DeviceRegisterResponse = device.clone().into();
        assert_eq!(resp.id, device.id);
        assert_eq!(resp.platform, device.platform);
        assert_eq!(resp.device_name, device.device_name);
        assert_eq!(resp.address, device.address);
        assert_eq!(resp.pairing_token, device.pairing_token);
    }

    /// UT-domain-06：DeviceRegisterRequest 缺 address 拒绝。
    #[test]
    fn register_request_missing_address_rejects() {
        let json = serde_json::json!({
            "device_id": uuid::Uuid::new_v4(),
            "platform": "desktop"
        });
        let result: Result<DeviceRegisterRequest, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    /// UT-domain-07：StatusReportRequest.device_name 可选。
    #[test]
    fn status_request_device_name_optional() {
        let json = serde_json::json!({
            "address": { "ip": "1.2.3.4", "port": 38425 }
        });
        let req: StatusReportRequest = serde_json::from_value(json).unwrap();
        assert!(req.device_name.is_none());
    }

    /// UT-domain-09：device_name 超 64 字符拒绝（Pre-req-10）。
    #[test]
    fn register_request_device_name_too_long_rejects() {
        let long_name = "a".repeat(65);
        let req = DeviceRegisterRequest {
            device_id: uuid::Uuid::new_v4(),
            platform: Platform::Desktop,
            device_name: Some(long_name),
            address: DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        assert!(req.validate().is_err());
    }

    /// UT-domain-10：device_name 恰好 64 字符通过。
    #[test]
    fn register_request_device_name_max_length_passes() {
        let name = "a".repeat(64);
        let req = DeviceRegisterRequest {
            device_id: uuid::Uuid::new_v4(),
            platform: Platform::Desktop,
            device_name: Some(name),
            address: DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        assert!(req.validate().is_ok());
    }

    /// UT-domain-11：device_name=None 通过。
    #[test]
    fn register_request_no_device_name_passes() {
        let req = DeviceRegisterRequest {
            device_id: uuid::Uuid::new_v4(),
            platform: Platform::Desktop,
            device_name: None,
            address: DeviceAddress {
                ip: "1.2.3.4".into(),
                port: 38425,
            },
        };
        assert!(req.validate().is_ok());
    }
}

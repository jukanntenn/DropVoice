use serde::Serialize;

/// Application error type.
///
/// Implements `serde::Serialize` so it can be returned from Tauri commands and
/// parsed by the frontend into `{ code, message, context }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Server already running")]
    ServerAlreadyRunning,

    #[error("Server start failed: {reason}")]
    ServerStartFailed { reason: String },

    #[error("Port {port} in use")]
    PortInUse { port: u16 },

    #[error("Failed to bind port: {reason}")]
    PortBindFailed { port: u16, reason: String },

    #[error("Network unreachable: {reason}")]
    NetworkUnreachable { reason: String },

    #[error("Connection refused")]
    ConnectionRefused,

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Text injection failed: {reason}")]
    TextInjectionFailed { reason: String },

    #[error("Text too long: {length}/{max} characters")]
    TextTooLong { length: usize, max: usize },

    #[error("Text is empty")]
    TextEmpty,

    #[error("Invalid language: {lang}")]
    InvalidLanguage { lang: String },

    #[error("Invalid theme: {theme}")]
    InvalidTheme { theme: String },

    #[error("Failed to save settings: {reason}")]
    SettingsSaveFailed { reason: String },

    #[error("Device not found: {device_id}")]
    DeviceNotFound { device_id: String },

    #[error("Device already connected: {device_id}")]
    DeviceAlreadyConnected { device_id: String },

    #[error("Maximum devices reached: {max}")]
    MaxDevicesReached { max: usize },

    #[error("Message too large: {length}/{max} bytes")]
    MessageTooLarge { length: usize, max: usize },

    #[error("Invalid message: {reason}")]
    MessageInvalid { reason: String },

    #[error("Injection queue full: {depth}/{max}")]
    QueueFull { depth: usize, max: usize },

    #[error("IO error: {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Machine-readable error code for frontend i18n mapping.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ServerAlreadyRunning => "SERVER_ALREADY_RUNNING",
            Self::ServerStartFailed { .. } => "SERVER_START_FAILED",
            Self::PortInUse { .. } => "PORT_IN_USE",
            Self::PortBindFailed { .. } => "PORT_BIND_FAILED",
            Self::NetworkUnreachable { .. } => "NETWORK_UNREACHABLE",
            Self::ConnectionRefused => "CONNECTION_REFUSED",
            Self::ConnectionTimeout => "CONNECTION_TIMEOUT",
            Self::ConnectionLost => "CONNECTION_LOST",
            Self::TextInjectionFailed { .. } => "TEXT_INJECTION_FAILED",
            Self::TextTooLong { .. } => "TEXT_TOO_LONG",
            Self::TextEmpty => "TEXT_EMPTY",
            Self::InvalidLanguage { .. } => "INVALID_LANGUAGE",
            Self::InvalidTheme { .. } => "INVALID_THEME",
            Self::SettingsSaveFailed { .. } => "SETTINGS_SAVE_FAILED",
            Self::DeviceNotFound { .. } => "DEVICE_NOT_FOUND",
            Self::DeviceAlreadyConnected { .. } => "DEVICE_ALREADY_CONNECTED",
            Self::MaxDevicesReached { .. } => "MAX_DEVICES_REACHED",
            Self::MessageTooLarge { .. } => "MESSAGE_TOO_LARGE",
            Self::MessageInvalid { .. } => "MESSAGE_INVALID",
            Self::QueueFull { .. } => "QUEUE_FULL",
            Self::Io { .. } => "IO_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Structured context for frontend display.
    pub fn error_context(&self) -> serde_json::Value {
        match self {
            Self::PortInUse { port } => serde_json::json!({ "port": port }),
            Self::ServerStartFailed { reason } => serde_json::json!({ "reason": reason }),
            Self::PortBindFailed { port, reason } => {
                serde_json::json!({ "port": port, "reason": reason })
            }
            Self::NetworkUnreachable { reason } => serde_json::json!({ "reason": reason }),
            Self::TextInjectionFailed { reason } => serde_json::json!({ "reason": reason }),
            Self::TextTooLong { length, max } => {
                serde_json::json!({ "length": length, "max": max })
            }
            Self::InvalidLanguage { lang } => serde_json::json!({ "lang": lang }),
            Self::InvalidTheme { theme } => serde_json::json!({ "theme": theme }),
            Self::SettingsSaveFailed { reason } => serde_json::json!({ "reason": reason }),
            Self::DeviceNotFound { device_id } => serde_json::json!({ "deviceId": device_id }),
            Self::DeviceAlreadyConnected { device_id } => {
                serde_json::json!({ "deviceId": device_id })
            }
            Self::MaxDevicesReached { max } => serde_json::json!({ "max": max }),
            Self::MessageTooLarge { length, max } => {
                serde_json::json!({ "length": length, "max": max })
            }
            Self::MessageInvalid { reason } => serde_json::json!({ "reason": reason }),
            Self::QueueFull { depth, max } => serde_json::json!({ "depth": depth, "max": max }),
            Self::Io { path, .. } => serde_json::json!({ "path": path }),
            Self::Internal(msg) => serde_json::json!({ "detail": msg }),
            _ => serde_json::json!({}),
        }
    }

    /// Optional user-facing suggestion key.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::NetworkUnreachable { .. } => Some("check_network"),
            Self::ConnectionRefused => Some("check_server_running"),
            Self::ConnectionTimeout => Some("retry_later"),
            Self::PortInUse { .. } => Some("change_port"),
            Self::MaxDevicesReached { .. } => Some("remove_device"),
            _ => None,
        }
    }
}

/// Result alias used by all Tauri commands.
pub type AppResult<T> = Result<T, AppError>;

/// Serializes the error as `{ code, message, context }` for the frontend.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_json = serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
            "context": self.error_context(),
        });
        error_json.serialize(serializer)
    }
}

impl AppError {
    /// Convenience constructor for IO errors with a path context.
    pub fn io(path: impl AsRef<std::path::Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().display().to_string(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_already_running_code_and_context() {
        let err = AppError::ServerAlreadyRunning;
        assert_eq!(err.error_code(), "SERVER_ALREADY_RUNNING");
        assert_eq!(err.error_context(), serde_json::json!({}));
        assert!(err.suggestion().is_none());
    }

    #[test]
    fn port_in_use_code_context_suggestion() {
        let err = AppError::PortInUse { port: 38425 };
        assert_eq!(err.error_code(), "PORT_IN_USE");
        assert_eq!(err.error_context(), serde_json::json!({ "port": 38425 }));
        assert_eq!(err.suggestion(), Some("change_port"));
    }

    #[test]
    fn port_bind_failed_code_and_context() {
        let err = AppError::PortBindFailed {
            port: 38425,
            reason: "permission denied".into(),
        };
        assert_eq!(err.error_code(), "PORT_BIND_FAILED");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "port": 38425, "reason": "permission denied" })
        );
    }

    #[test]
    fn network_unreachable_has_suggestion() {
        let err = AppError::NetworkUnreachable {
            reason: "offline".into(),
        };
        assert_eq!(err.error_code(), "NETWORK_UNREACHABLE");
        assert_eq!(err.suggestion(), Some("check_network"));
    }

    #[test]
    fn connection_variants_codes() {
        assert_eq!(
            AppError::ConnectionRefused.error_code(),
            "CONNECTION_REFUSED"
        );
        assert_eq!(
            AppError::ConnectionTimeout.error_code(),
            "CONNECTION_TIMEOUT"
        );
        assert_eq!(AppError::ConnectionLost.error_code(), "CONNECTION_LOST");
    }

    #[test]
    fn text_injection_failed_context() {
        let err = AppError::TextInjectionFailed {
            reason: "enigo error".into(),
        };
        assert_eq!(err.error_code(), "TEXT_INJECTION_FAILED");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "reason": "enigo error" })
        );
    }

    #[test]
    fn text_too_long_context() {
        let err = AppError::TextTooLong {
            length: 20000,
            max: 10000,
        };
        assert_eq!(err.error_code(), "TEXT_TOO_LONG");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "length": 20000, "max": 10000 })
        );
    }

    #[test]
    fn text_empty_code() {
        assert_eq!(AppError::TextEmpty.error_code(), "TEXT_EMPTY");
    }

    #[test]
    fn invalid_language_and_theme_context() {
        let lang_err = AppError::InvalidLanguage { lang: "fr".into() };
        assert_eq!(lang_err.error_code(), "INVALID_LANGUAGE");
        assert_eq!(
            lang_err.error_context(),
            serde_json::json!({ "lang": "fr" })
        );

        let theme_err = AppError::InvalidTheme {
            theme: "purple".into(),
        };
        assert_eq!(theme_err.error_code(), "INVALID_THEME");
        assert_eq!(
            theme_err.error_context(),
            serde_json::json!({ "theme": "purple" })
        );
    }

    #[test]
    fn settings_save_failed_context() {
        let err = AppError::SettingsSaveFailed {
            reason: "disk full".into(),
        };
        assert_eq!(err.error_code(), "SETTINGS_SAVE_FAILED");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "reason": "disk full" })
        );
    }

    #[test]
    fn device_variants_codes_and_context() {
        let not_found = AppError::DeviceNotFound {
            device_id: "dev1".into(),
        };
        assert_eq!(not_found.error_code(), "DEVICE_NOT_FOUND");
        assert_eq!(
            not_found.error_context(),
            serde_json::json!({ "deviceId": "dev1" })
        );

        let already = AppError::DeviceAlreadyConnected {
            device_id: "dev2".into(),
        };
        assert_eq!(already.error_code(), "DEVICE_ALREADY_CONNECTED");

        let maxed = AppError::MaxDevicesReached { max: 5 };
        assert_eq!(maxed.error_code(), "MAX_DEVICES_REACHED");
        assert_eq!(maxed.suggestion(), Some("remove_device"));
    }

    #[test]
    fn message_too_large_code_and_context() {
        let err = AppError::MessageTooLarge {
            length: 2_000_000,
            max: 1_000_000,
        };
        assert_eq!(err.error_code(), "MESSAGE_TOO_LARGE");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "length": 2_000_000, "max": 1_000_000 })
        );
    }

    #[test]
    fn message_invalid_code_and_context() {
        let err = AppError::MessageInvalid {
            reason: "bad json".into(),
        };
        assert_eq!(err.error_code(), "MESSAGE_INVALID");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "reason": "bad json" })
        );
    }

    #[test]
    fn queue_full_code_and_context() {
        let err = AppError::QueueFull {
            depth: 100,
            max: 100,
        };
        assert_eq!(err.error_code(), "QUEUE_FULL");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "depth": 100, "max": 100 })
        );
    }

    #[test]
    fn io_error_code_and_context() {
        let err = AppError::Io {
            path: "/tmp/config.toml".into(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        };
        assert_eq!(err.error_code(), "IO_ERROR");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "path": "/tmp/config.toml" })
        );
    }

    #[test]
    fn internal_error_code_and_context() {
        let err = AppError::Internal("unexpected".into());
        assert_eq!(err.error_code(), "INTERNAL_ERROR");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "detail": "unexpected" })
        );
    }

    #[test]
    fn serialize_produces_code_message_context() {
        let err = AppError::PortInUse { port: 38425 };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "PORT_IN_USE");
        assert!(json["message"].as_str().unwrap().contains("38425"));
        assert_eq!(json["context"]["port"], 38425);
    }
}

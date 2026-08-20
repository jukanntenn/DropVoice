//! 凭据生成与校验（webrtc-scan-direct-design §3 凭据体系）。
//!
//! - Pairing Code：6 位数字（19.9 bit 熵），桌面生成 + 桌面本地验证（§3.2）。
//! - Connection Token：`dvct_` + 43 字符 base64url（256 bit），替代 UUID v4（§3.3）。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;

use crate::config::ConnectionTokenRecord;

/// 连接令牌 TTL：签发后 30 天内有效，之后手机需重新配对（重新扫码）。
///
/// 令牌持久化在 config.toml（`device.connected_tokens`）——重启桌面不丢，
/// 但过期即失效，避免令牌永续持有（与 5min 配对码形成时间梯度）。
pub const CONNECTION_TOKEN_TTL: Duration = Duration::days(30);

/// 生成 6 位数字配对码（19.9 bit 熵，§3.2）。
///
/// code 只在 QR 中和桌面屏幕上出现，从不在公网传输——它随 SDP offer 走信令
/// 服务器时，校验发生在桌面本地。5min TTL 内同一 code 可被多次使用（多手机同时配对）。
pub fn generate_pairing_code() -> String {
    let mut rng = rand::thread_rng();
    // 0..1_000_000 均匀分布，前导零补齐到 6 位。
    let n: u32 = rng.gen_range(0..1_000_000);
    format!("{n:06}")
}

/// Verifies a pairing code request against the stored code and its issue time.
///
/// Returns `true` only when the code matches **and** has not expired (§3.2 5min TTL）。
pub fn verify_pairing_code(
    stored: &str,
    stored_time: DateTime<Utc>,
    request: &str,
    expiry_minutes: u64,
) -> bool {
    if stored != request {
        return false;
    }
    let expiry = Duration::minutes(expiry_minutes as i64);
    Utc::now().signed_duration_since(stored_time) <= expiry
}

/// 生成 `dvct_` 连接令牌（256 bit 熵，§3.3）。
///
/// 格式：`dvct_` + 43 字符 base64url（URL_SAFE_NO_PAD 编码 32 随机字节）。
/// `dvct` = DropVoice Connection Token。用 CSPRNG（rand::Rng::fill）。
pub fn generate_connection_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    format!("dvct_{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// Verifies a connection token against the list of issued tokens (with TTL).
///
/// `stored` 是持久化的令牌记录；token 必须存在**且**未超过 30 天 TTL。
pub fn verify_connection_token(token: &str, stored: &[ConnectionTokenRecord]) -> bool {
    let now = Utc::now();
    stored
        .iter()
        .any(|r| r.token == token && now.signed_duration_since(r.issued_at) <= CONNECTION_TOKEN_TTL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_code_is_six_digits() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn pairing_code_supports_leading_zeros() {
        // 多次生成，确保 6 位长度（前导零补齐）始终成立。
        for _ in 0..200 {
            let code = generate_pairing_code();
            assert_eq!(code.len(), 6, "code was {code}");
        }
    }

    #[test]
    fn pairing_code_in_valid_range() {
        // 解析为整数，应在 0..1_000_000 范围内。
        let code = generate_pairing_code();
        let n: u32 = code.parse().unwrap();
        assert!(n < 1_000_000);
    }

    #[test]
    fn verify_correct_pairing_code_within_expiry() {
        let code = "123456";
        let issued = Utc::now();
        assert!(verify_pairing_code(code, issued, code, 5));
    }

    #[test]
    fn reject_wrong_pairing_code() {
        let issued = Utc::now();
        assert!(!verify_pairing_code("123456", issued, "654321", 5));
    }

    #[test]
    fn reject_expired_pairing_code() {
        let issued = Utc::now() - Duration::minutes(10);
        assert!(!verify_pairing_code("123456", issued, "123456", 5));
    }

    #[test]
    fn connection_token_has_dvct_prefix() {
        let token = generate_connection_token();
        assert!(token.starts_with("dvct_"));
    }

    #[test]
    fn connection_token_is_43_chars_after_prefix() {
        // 32 字节 base64url 无 padding = ceil(32/3)*4 = 44 → 实际 43（无 padding）。
        let token = generate_connection_token();
        let body = &token["dvct_".len()..];
        assert_eq!(body.len(), 43, "token body was {body}");
    }

    #[test]
    fn connection_token_is_url_safe() {
        // base64url 字母表：A-Za-z0-9-_，无 + / =。
        let token = generate_connection_token();
        let body = &token["dvct_".len()..];
        assert!(
            body.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "token body contained non-url-safe char: {body}"
        );
        assert!(!body.contains('='));
    }

    #[test]
    fn connection_token_has_256_bit_entropy() {
        // 32 字节 = 256 bit。通过两次生成不同验证随机性。
        let t1 = generate_connection_token();
        let t2 = generate_connection_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn verify_issued_token() {
        let token = generate_connection_token();
        let stored = vec![ConnectionTokenRecord {
            token: token.clone(),
            issued_at: Utc::now(),
        }];
        assert!(verify_connection_token(&token, &stored));
    }

    #[test]
    fn reject_unknown_token() {
        let stored = vec![ConnectionTokenRecord {
            token: "dvct_abc".into(),
            issued_at: Utc::now(),
        }];
        assert!(!verify_connection_token("dvct_xyz", &stored));
    }

    #[test]
    fn reject_expired_connection_token() {
        let token = generate_connection_token();
        let stored = vec![ConnectionTokenRecord {
            token: token.clone(),
            issued_at: Utc::now() - CONNECTION_TOKEN_TTL - Duration::days(1),
        }];
        assert!(!verify_connection_token(&token, &stored));
    }

    #[test]
    fn token_within_ttl_is_valid() {
        let token = generate_connection_token();
        let stored = vec![ConnectionTokenRecord {
            token: token.clone(),
            issued_at: Utc::now() - CONNECTION_TOKEN_TTL + Duration::minutes(1),
        }];
        assert!(verify_connection_token(&token, &stored));
    }
}

//! API 辅助：随机凭据生成。
//!
//! pairing_token（64 字符，服务端颁发给桌面）。使用 `rand` 的 thread RNG。
//! 配对码（code）改由桌面生成 + 桌面验证（webrtc-scan-direct-design §3.2），
//! 服务端零 code 知识，故此处不再生成 code。

use crate::config::PAIRING_TOKEN_LEN;

const TOKEN_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// 生成 64 字符随机 token（pairing_token，桌面↔服务器认证）。
pub fn generate_token() -> String {
    random_string(PAIRING_TOKEN_LEN, TOKEN_ALPHABET)
}

fn random_string(len: usize, alphabet: &[u8]) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..alphabet.len());
            alphabet[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UT-gen-01：token 长度 64。
    #[test]
    fn token_length_is_64() {
        assert_eq!(generate_token().len(), PAIRING_TOKEN_LEN);
    }

    /// UT-gen-02：token 字符集限定在 TOKEN_ALPHABET。
    #[test]
    fn token_charset_limited() {
        let token = generate_token();
        assert!(token.chars().all(|c| TOKEN_ALPHABET.contains(&(c as u8))));
    }

    /// UT-gen-03：两次生成不同（随机性）。
    #[test]
    fn token_is_random() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
    }
}

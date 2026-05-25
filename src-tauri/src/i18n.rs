// src-tauri/src/i18n.rs
use std::env;

pub struct TrayTexts {
    pub show: &'static str,
    pub hide: &'static str,
    pub quit: &'static str,
}

pub fn validate_language(lang: &str) -> Result<&str, String> {
    match lang {
        "en" | "zh" | "zh-TW" | "ja" => Ok(lang),
        _ => Err("Invalid language code. Must be 'en', 'zh', 'zh-TW', or 'ja'".to_string()),
    }
}

fn detect_from_locale(locale: &str) -> Option<String> {
    let lower = locale.to_lowercase();
    if lower.starts_with("zh-tw") || lower.starts_with("zh-hk") || lower.starts_with("zh-mo") || lower.starts_with("zh-hant") {
        return Some("zh-TW".to_string());
    }
    if lower.starts_with("zh") {
        return Some("zh".to_string());
    }
    if lower.starts_with("ja") {
        return Some("ja".to_string());
    }
    None
}

pub fn get_system_language() -> String {
    // Check LANG environment variable (Unix/Linux/macOS)
    if let Ok(lang) = env::var("LANG") {
        if let Some(detected) = detect_from_locale(&lang) {
            return detected;
        }
    }

    // Windows-specific: Check environment variables
    if let Ok(lang) = env::var("LANGUAGE") {
        if let Some(detected) = detect_from_locale(&lang) {
            return detected;
        }
    }

    "en".to_string()
}

pub fn get_tray_texts(lang: &str) -> TrayTexts {
    match lang {
        "zh" => TrayTexts {
            show: "显示",
            hide: "隐藏",
            quit: "退出",
        },
        "zh-TW" => TrayTexts {
            show: "顯示",
            hide: "隱藏",
            quit: "結束",
        },
        "ja" => TrayTexts {
            show: "表示",
            hide: "非表示",
            quit: "終了",
        },
        _ => TrayTexts {
            show: "Show",
            hide: "Hide",
            quit: "Quit",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_language_accepts_en() {
        assert_eq!(validate_language("en"), Ok("en"));
    }

    #[test]
    fn test_validate_language_accepts_zh() {
        assert_eq!(validate_language("zh"), Ok("zh"));
    }

    #[test]
    fn test_validate_language_accepts_zh_tw() {
        assert_eq!(validate_language("zh-TW"), Ok("zh-TW"));
    }

    #[test]
    fn test_validate_language_accepts_ja() {
        assert_eq!(validate_language("ja"), Ok("ja"));
    }

    #[test]
    fn test_validate_language_rejects_invalid() {
        assert!(validate_language("fr").is_err());
        assert!(validate_language("").is_err());
    }

    #[test]
    fn test_get_system_language_returns_valid() {
        let lang = get_system_language();
        assert!(lang == "en" || lang == "zh" || lang == "zh-TW" || lang == "ja");
    }

    #[test]
    fn test_detect_from_locale() {
        assert_eq!(detect_from_locale("zh-TW"), Some("zh-TW".to_string()));
        assert_eq!(detect_from_locale("zh-HK"), Some("zh-TW".to_string()));
        assert_eq!(detect_from_locale("zh-Hant"), Some("zh-TW".to_string()));
        assert_eq!(detect_from_locale("zh-CN"), Some("zh".to_string()));
        assert_eq!(detect_from_locale("zh"), Some("zh".to_string()));
        assert_eq!(detect_from_locale("ja"), Some("ja".to_string()));
        assert_eq!(detect_from_locale("ja-JP"), Some("ja".to_string()));
        assert_eq!(detect_from_locale("en"), None);
        assert_eq!(detect_from_locale("en-US"), None);
    }

    #[test]
    fn test_tray_texts_en() {
        let texts = get_tray_texts("en");
        assert_eq!(texts.show, "Show");
        assert_eq!(texts.hide, "Hide");
        assert_eq!(texts.quit, "Quit");
    }

    #[test]
    fn test_tray_texts_zh() {
        let texts = get_tray_texts("zh");
        assert_eq!(texts.show, "显示");
        assert_eq!(texts.hide, "隐藏");
        assert_eq!(texts.quit, "退出");
    }

    #[test]
    fn test_tray_texts_zh_tw() {
        let texts = get_tray_texts("zh-TW");
        assert_eq!(texts.show, "顯示");
        assert_eq!(texts.hide, "隱藏");
        assert_eq!(texts.quit, "結束");
    }

    #[test]
    fn test_tray_texts_ja() {
        let texts = get_tray_texts("ja");
        assert_eq!(texts.show, "表示");
        assert_eq!(texts.hide, "非表示");
        assert_eq!(texts.quit, "終了");
    }
}

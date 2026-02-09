// src-tauri/src/i18n.rs
use std::env;

pub struct TrayTexts {
    pub show: &'static str,
    pub hide: &'static str,
    pub quit: &'static str,
}

pub fn validate_language(lang: &str) -> Result<&str, String> {
    match lang {
        "en" | "zh" => Ok(lang),
        _ => Err("Invalid language code. Must be 'en' or 'zh'".to_string()),
    }
}

pub fn get_system_language() -> String {
    // Check LANG environment variable (Unix/Linux/macOS)
    if let Ok(lang) = env::var("LANG") {
        if lang.starts_with("zh") {
            return "zh".to_string();
        }
    }

    // Windows-specific: Check environment variables
    if let Ok(lang) = env::var("LANGUAGE") {
        if lang.starts_with("zh") {
            return "zh".to_string();
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
    fn test_validate_language_rejects_invalid() {
        assert!(validate_language("fr").is_err());
        assert!(validate_language("zh-CN").is_err());
        assert!(validate_language("").is_err());
    }

    #[test]
    fn test_get_system_language_returns_valid() {
        let lang = get_system_language();
        assert!(lang == "en" || lang == "zh");
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
}

//! Backend i18n support (spec 15 §5 constraint 5).
//!
//! Provides translations for system tray menu items and other backend strings.
//! The Rust backend embeds a minimal set of translations at compile time rather
//! than reading from `packages/i18n/locales/` at runtime (the backend doesn't
//! have direct access to the JS i18n bundle).
//!
//! Languages supported: `en`, `zh`, `zh-TW`, `ja` (spec 15 §2.1).

use serde::Deserialize;

/// Translated strings for the system tray context menu (spec 15 §2.6).
#[derive(Debug, Clone, Deserialize)]
pub struct TrayTranslations {
    pub show: String,
    pub hide: String,
    pub quit: String,
}

/// Loads tray menu translations for the given language code.
///
/// Falls back to English for unknown languages (spec 15 §2.5 constraint 4).
pub fn load_tray_translations(lang: &str) -> TrayTranslations {
    match lang {
        "zh" => TrayTranslations {
            show: "显示".into(),
            hide: "隐藏".into(),
            quit: "退出".into(),
        },
        "zh-TW" => TrayTranslations {
            show: "顯示".into(),
            hide: "隱藏".into(),
            quit: "結束".into(),
        },
        "ja" => TrayTranslations {
            show: "表示".into(),
            hide: "非表示".into(),
            quit: "終了".into(),
        },
        _ => TrayTranslations {
            show: "Show".into(),
            hide: "Hide".into(),
            quit: "Quit".into(),
        },
    }
}

/// Detects the system language from the locale.
///
/// Returns a 2-letter code (`en`, `zh`, `ja`) or `en` as fallback.
pub fn get_system_language() -> String {
    // Try environment variables first (works cross-platform).
    for var in ["LANG", "LC_ALL", "LC_MESSAGES", "LANGUAGE"] {
        if let Ok(val) = std::env::var(var) {
            let lower = val.to_lowercase();
            if lower.starts_with("zh-tw") || lower.starts_with("zh_hant") {
                return "zh-TW".into();
            }
            if lower.starts_with("zh") {
                return "zh".into();
            }
            if lower.starts_with("ja") {
                return "ja".into();
            }
            if lower.starts_with("en") {
                return "en".into();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: check the UI language via the `sys-locale` approach.
        // For now, fall back to "en" — the config file takes precedence.
        "en".into()
    }

    #[cfg(not(target_os = "windows"))]
    {
        "en".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_translations() {
        let t = load_tray_translations("en");
        assert_eq!(t.show, "Show");
        assert_eq!(t.hide, "Hide");
        assert_eq!(t.quit, "Quit");
    }

    #[test]
    fn chinese_translations() {
        let t = load_tray_translations("zh");
        assert_eq!(t.show, "显示");
        assert_eq!(t.hide, "隐藏");
        assert_eq!(t.quit, "退出");
    }

    #[test]
    fn chinese_traditional_translations() {
        let t = load_tray_translations("zh-TW");
        assert_eq!(t.show, "顯示");
        assert_eq!(t.hide, "隱藏");
        assert_eq!(t.quit, "結束");
    }

    #[test]
    fn japanese_translations() {
        let t = load_tray_translations("ja");
        assert_eq!(t.show, "表示");
        assert_eq!(t.hide, "非表示");
        assert_eq!(t.quit, "終了");
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        let t = load_tray_translations("fr");
        assert_eq!(t.show, "Show");
        assert_eq!(t.hide, "Hide");
        assert_eq!(t.quit, "Quit");
    }

    #[test]
    fn get_system_language_returns_valid_code() {
        let lang = get_system_language();
        assert!(
            ["en", "zh", "zh-TW", "ja"].contains(&lang.as_str()),
            "unexpected language: {lang}"
        );
    }
}

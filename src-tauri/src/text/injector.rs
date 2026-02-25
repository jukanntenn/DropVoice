use anyhow::Result;
use enigo::{Enigo, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Inject text at the current cursor position by simulating keyboard input
///
/// Uses buffered character-by-character input with configurable delay to ensure
/// complete text injection on all platforms, especially Windows.
pub fn inject_text(text: &str, delay_ms: u64) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;

    // For each character, inject with delay
    for c in text.chars() {
        enigo.text(&c.to_string())?;
        thread::sleep(Duration::from_millis(delay_ms));
    }

    Ok(())
}

/// Inject text without delay (legacy function for backward compatibility)
///
/// Note: This may result in incomplete input on Windows. Use inject_text with delay instead.
pub fn inject_text_fast(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_simple_text() {
        let result = inject_text("Hello, World!", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_newlines() {
        let result = inject_text("Line 1\nLine 2\nLine 3", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_unicode() {
        let result = inject_text("Hello 世界 🦀", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_chinese_punctuation() {
        let result = inject_text("测试中文标点：逗号，句号。问号？感叹！", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_empty_string() {
        let result = inject_text("", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_fast_mode() {
        let result = inject_text_fast("Hello, World!");
        assert!(result.is_ok());
    }
}

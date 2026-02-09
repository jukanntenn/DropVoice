use anyhow::Result;
use enigo::{Enigo, Keyboard, Settings};

/// Inject text at the current cursor position by simulating keyboard input
///
/// Uses enigo's batch input mode which sends all characters in a single
/// SendInput call, reducing IME interference on Windows.
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_simple_text() {
        let result = inject_text("Hello, World!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_newlines() {
        let result = inject_text("Line 1\nLine 2\nLine 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_unicode() {
        let result = inject_text("Hello 世界 🦀");
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_chinese_punctuation() {
        let result = inject_text("测试中文标点：逗号，句号。问号？感叹！");
        assert!(result.is_ok());
    }
}

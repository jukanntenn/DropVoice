use crate::error::{AppError, AppResult};

#[cfg(test)]
use std::sync::Mutex;

/// 单次注入文本上限的默认值（`injection.max_text_length` 的 serde 默认）。
/// 实际生效值来自 config.toml，由调用方传入 `validate_text` / `Injector::inject`。
pub const DEFAULT_MAX_TEXT_LENGTH: usize = 10000;

/// Validates text without performing any injection.
///
/// Pure function — no side effects, safe to call from any context. Returns a
/// structured `AppError` for empty / too-long input so the frontend can display
/// a localised message. Extracted from the original `inject_text` so the
/// boundary checks are testable in isolation (spec 06 §2.3 — keep validation
/// out of the side-effecting path).
pub fn validate_text(text: &str, max_length: usize) -> AppResult<()> {
    if text.is_empty() {
        return Err(AppError::TextEmpty);
    }
    let char_count = text.chars().count();
    if char_count > max_length {
        return Err(AppError::TextTooLong {
            length: char_count,
            max: max_length,
        });
    }
    Ok(())
}

/// The single injection seam (spec 06 §2.3 — mock boundary dependencies).
///
/// Production wires in [`EnigoInjector`] (real keyboard injection via `enigo`);
/// tests wire in [`MockInjector`] so the test suite never touches the OS input
/// stack. This is the abstraction enigo itself does not provide: `enigo`'s own
/// tests perform real injection ("we assume it worked as long as there was no
/// panic", `enigo/src/tests/keyboard.rs`), leaving the responsibility for a
/// mockable boundary to the caller.
pub trait Injector: Send + Sync {
    /// Validates `text` (against `max_text_length`), then injects it at the
    /// current cursor position.
    fn inject(&self, text: &str, delay_ms: u64, max_text_length: usize) -> AppResult<()>;
}

/// Production [`Injector`] backed by `enigo` (spec 11 — LAN text injection).
///
/// Behaviour is byte-for-byte equivalent to the historical module-level
/// `inject_text`: validate first, then either fast-path the whole string via
/// `Keyboard::text` (when `delay_ms == 0`) or replay the per-character loop
/// with a sleep to honour the configured inter-keystroke delay.
pub struct EnigoInjector;

impl EnigoInjector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnigoInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl Injector for EnigoInjector {
    fn inject(&self, text: &str, delay_ms: u64, max_text_length: usize) -> AppResult<()> {
        validate_text(text, max_text_length)?;
        // 换行归一化：\r\n 与孤立 \r 统一为 \n（跨平台一致；Windows 注入器再把
        // \n 映射为字面 CR 字符事件，见 win32::inject_unicode）。
        let normalized = normalize_newlines(text);
        inject_with_enigo(&normalized, delay_ms)
    }
}

/// 换行归一化：`\r\n` 与孤立 `\r` 均归一为 `\n`。
///
/// 纯函数（可单测）。移动端 textarea 用 LF，但防御外部/历史数据可能携带 CRLF。
pub fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// 控制字符 → 字面 UNICODE 码元映射（Windows 注入器用）。
///
/// 关键事实（实证 + enigo 0.6.1 源码）：
/// - `VkKeyScanExW` 把 `\n`/`\r` 都映射到 `VK_RETURN`（keycodes.rs:1051-1057）→
///   走 enigo 必然产生真回车键。`KEYEVENTF_UNICODE` 直接送 U+000D 则生成
///   `WM_CHAR(0x0D)`——编辑控件识别为换行，且全程无 VK_RETURN 键事件。
/// - `\t` 同理：`VkKeyScanExW(0x09)=VK_TAB`；字面 U+0009 事件在编辑控件内插入制表。
/// - 其余控制字符无法以字面形式注入 → `None`（调用方跳过并告警）。
pub fn control_char_unicode(c: char) -> Option<u16> {
    match c {
        '\n' | '\r' => Some(0x000D),
        '\t' => Some(0x0009),
        _ if c.is_control() => None,
        _ => None,
    }
}

/// 平台注入分发。
///
/// - Windows：统一 `KEYEVENTF_UNICODE` 字面注入（布局无关、CJK 天然正确、
///   换行/制表原样；本机 WinForms+低级钩子实证：换行以 WM_CHAR(0x0D) 到达，
///   零 VK_RETURN 键事件）。
/// - macOS：普通字符走 enigo.key；控制字符走 enigo.text()（fast_text 的
///   `keyboard_set_unicode_string` 保换行，含前导换行 workaround，enigo#261）。
/// - Linux：保持 enigo 逐字符 keysym 路径（Wayland/libei 直传 Linefeed keysym
///   正确；X11 上 keysym 经 keycode 往返会变 Return —— 平台限制，见 PRINCIPLES）。
fn inject_with_enigo(text: &str, delay_ms: u64) -> AppResult<()> {
    #[cfg(windows)]
    {
        win32::inject_unicode(text, delay_ms)
    }
    #[cfg(not(windows))]
    {
        inject_with_enigo_unix(text, delay_ms)
    }
}

/// Windows 后端：SendInput(KEYEVENTF_UNICODE) 逐字符字面注入。
#[cfg(windows)]
mod win32 {
    use std::time::Duration;

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    };

    use crate::error::{AppError, AppResult};

    /// 逐字符注入 UNICODE 键盘事件（KEYEVENTF_UNICODE，VK_PACKET 语义）。
    ///
    /// 每个字符（UTF-16 码元）发送 down+up 两条事件；`delay_ms` 为字符间隔。
    pub fn inject_unicode(text: &str, delay_ms: u64) -> AppResult<()> {
        let per_char = Duration::from_millis(delay_ms);
        for c in text.chars() {
            if c.is_control() {
                match super::control_char_unicode(c) {
                    Some(unit) => send_units(&[unit])?,
                    None => {
                        tracing::warn!(
                            char = ?c,
                            code = c as u32,
                            "skipping control character (not injectable as literal text)"
                        );
                        continue;
                    }
                }
            } else {
                let mut buf = [0u16; 2];
                let units = c.encode_utf16(&mut buf);
                send_units(units)?;
            }
            if delay_ms > 0 {
                std::thread::sleep(per_char);
            }
        }
        Ok(())
    }

    fn send_units(units: &[u16]) -> AppResult<()> {
        for &unit in units {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: 0,
                            wScan: unit,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: 0,
                            wScan: unit,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            // SAFETY: inputs 布局与 win32 INPUT（type + union）逐字段一致；
            // 由 windows-sys 生成的绑定保证。
            let sent = unsafe {
                SendInput(
                    inputs.len() as u32,
                    inputs.as_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                )
            };
            if sent != 2 {
                return Err(AppError::TextInjectionFailed {
                    reason: format!("SendInput returned {sent} (expected 2), unit U+{unit:04X}"),
                });
            }
        }
        Ok(())
    }
}

/// macOS / Linux 后端：enigo 逐字符（控制字符走 text() 保字面换行）。
#[cfg(not(windows))]
fn inject_with_enigo_unix(text: &str, delay_ms: u64) -> AppResult<()> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    use std::thread::sleep;
    use std::time::Duration;

    let settings = Settings::default();
    let mut enigo = Enigo::new(&settings).map_err(|e| AppError::TextInjectionFailed {
        reason: e.to_string(),
    })?;

    if delay_ms == 0 {
        enigo
            .text(text)
            .map_err(|e| AppError::TextInjectionFailed {
                reason: e.to_string(),
            })?;
        return Ok(());
    }

    let per_char = Duration::from_millis(delay_ms);
    for c in text.chars() {
        if c.is_control() {
            match c {
                '\n' | '\r' | '\t' => {
                    // macOS fast_text 保留字面换行/制表；Linux 走 enigo keysym 路径
                    //（X11 上换行映射为 Return，平台限制，见模块注释）。
                    enigo
                        .text(&c.to_string())
                        .map_err(|e| AppError::TextInjectionFailed {
                            reason: format!("control char {c:?}: {e}"),
                        })?;
                }
                _ => {
                    tracing::warn!(
                        char = ?c,
                        code = c as u32,
                        "skipping control character (not injectable as literal text)"
                    );
                    continue;
                }
            }
        } else {
            enigo.key(Key::Unicode(c), Direction::Click).map_err(|e| {
                AppError::TextInjectionFailed {
                    reason: format!("character U+{:04X} ({c:?}): {e}", c as u32),
                }
            })?;
        }
        sleep(per_char);
    }

    Ok(())
}

/// Hand-written mock [`Injector`] for tests (spec 06 §2.3 — minimal mock; the
/// project does not depend on `mockall`).
///
/// Records every `inject` call's `(text, delay_ms, max_text_length)` so tests
/// can assert that the queue processor dispatched the expected payloads
/// without touching the OS input stack. Validation still runs first, so
/// passing oversized/empty text surfaces the same `AppError` as the
/// production path.
#[cfg(test)]
#[derive(Default)]
pub struct MockInjector {
    pub calls: Mutex<Vec<(String, u64, usize)>>,
}

#[cfg(test)]
impl MockInjector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of recorded calls (text, delay_ms, max_text_length) in order.
    pub fn recorded(&self) -> Vec<(String, u64, usize)> {
        self.calls.lock().expect("mock calls lock poisoned").clone()
    }
}

#[cfg(test)]
impl Injector for MockInjector {
    fn inject(&self, text: &str, delay_ms: u64, max_text_length: usize) -> AppResult<()> {
        validate_text(text, max_text_length)?;
        self.calls.lock().expect("mock calls lock poisoned").push((
            text.to_string(),
            delay_ms,
            max_text_length,
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_text_rejects_empty() {
        let err = validate_text("", DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
    }

    #[test]
    fn validate_text_rejects_too_long() {
        let long = "a".repeat(DEFAULT_MAX_TEXT_LENGTH + 1);
        let err = validate_text(&long, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_TOO_LONG");
        assert_eq!(
            err.error_context(),
            serde_json::json!({
                "length": DEFAULT_MAX_TEXT_LENGTH + 1,
                "max": DEFAULT_MAX_TEXT_LENGTH
            })
        );
    }

    /// 上限来自 config.toml：非默认上限同样生效。
    #[test]
    fn validate_text_honors_configured_limit() {
        let err = validate_text("abc", 2).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_TOO_LONG");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "length": 3, "max": 2 })
        );
        validate_text("abc", 3).unwrap();
    }

    #[test]
    fn validate_text_accepts_exactly_max_length() {
        // Pure validation — no side effect, no enigo. This replaces the former
        // test that called inject_text with 10000 'a's and let enigo actually
        // type them into the focused window.
        let exact = "a".repeat(DEFAULT_MAX_TEXT_LENGTH);
        validate_text(&exact, DEFAULT_MAX_TEXT_LENGTH).unwrap();
    }

    #[test]
    fn mock_injector_records_calls_and_runs_validation() {
        let mock = MockInjector::new();
        mock.inject("hello", 10, DEFAULT_MAX_TEXT_LENGTH).unwrap();
        mock.inject("world", 0, DEFAULT_MAX_TEXT_LENGTH).unwrap();
        assert_eq!(
            mock.recorded(),
            vec![
                ("hello".to_string(), 10, DEFAULT_MAX_TEXT_LENGTH),
                ("world".to_string(), 0, DEFAULT_MAX_TEXT_LENGTH)
            ]
        );
    }

    #[test]
    fn mock_injector_surfaces_validation_errors() {
        let mock = MockInjector::new();
        let err = mock.inject("", 0, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
        // Validation failure must not record a call.
        assert!(mock.recorded().is_empty());
    }

    #[test]
    fn enigo_injector_validates_before_delegating() {
        // EnigoInjector's validation path is exercised without depending on a
        // display server: empty/oversized input returns before enigo is built.
        let inj = EnigoInjector::new();
        let err = inj.inject("", 0, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
        let long = "a".repeat(DEFAULT_MAX_TEXT_LENGTH + 1);
        let err = inj.inject(&long, 0, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_TOO_LONG");
    }

    #[test]
    fn inject_text_error_implements_send_sync() {
        // Compile-time check that AppError is Send + Sync (required for
        // returning from Tauri commands on a different thread).
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AppError>();
    }

    #[test]
    fn default_max_text_length_is_ten_thousand() {
        assert_eq!(DEFAULT_MAX_TEXT_LENGTH, 10_000);
    }

    /// 换行归一化：CRLF / 孤立 CR → LF；连续 CR 各成一行。
    #[test]
    fn normalize_newlines_unifies_cr_and_crlf() {
        assert_eq!(normalize_newlines("a\r\nb"), "a\nb");
        assert_eq!(normalize_newlines("a\rb"), "a\nb");
        assert_eq!(normalize_newlines("a\nb"), "a\nb");
        assert_eq!(normalize_newlines("a\r\rb"), "a\n\nb");
        assert_eq!(normalize_newlines("no newline"), "no newline");
        assert_eq!(normalize_newlines(""), "");
    }

    /// 控制字符 → 字面 UNICODE 码元映射（Windows 注入核心）。
    #[test]
    fn control_char_unicode_maps_newline_and_tab() {
        assert_eq!(control_char_unicode('\n'), Some(0x000D));
        assert_eq!(control_char_unicode('\r'), Some(0x000D));
        assert_eq!(control_char_unicode('\t'), Some(0x0009));
        // 其他控制字符无法字面注入。
        assert_eq!(control_char_unicode('\0'), None);
        assert_eq!(control_char_unicode('\u{0007}'), None);
        // 可打印字符不属于控制字符映射。
        assert_eq!(control_char_unicode('a'), None);
    }
}

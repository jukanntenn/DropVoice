use crate::error::{AppError, AppResult};

#[cfg(test)]
use std::sync::Mutex;

/// Maximum text length accepted by the injector (safety net — the server also
/// validates before enqueuing).
pub const MAX_TEXT_LENGTH: usize = 10000;

/// Validates text without performing any injection.
///
/// Pure function — no side effects, safe to call from any context. Returns a
/// structured `AppError` for empty / too-long input so the frontend can display
/// a localised message. Extracted from the original `inject_text` so the
/// boundary checks are testable in isolation (spec 06 §2.3 — keep validation
/// out of the side-effecting path).
pub fn validate_text(text: &str) -> AppResult<()> {
    if text.is_empty() {
        return Err(AppError::TextEmpty);
    }
    let char_count = text.chars().count();
    if char_count > MAX_TEXT_LENGTH {
        return Err(AppError::TextTooLong {
            length: char_count,
            max: MAX_TEXT_LENGTH,
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
    /// Validates `text`, then injects it at the current cursor position.
    fn inject(&self, text: &str, delay_ms: u64) -> AppResult<()>;
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
    fn inject(&self, text: &str, delay_ms: u64) -> AppResult<()> {
        validate_text(text)?;
        inject_with_enigo(text, delay_ms)
    }
}

fn inject_with_enigo(text: &str, delay_ms: u64) -> AppResult<()> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    use std::thread::sleep;
    use std::time::Duration;

    let settings = Settings::default();
    let mut enigo = Enigo::new(&settings).map_err(|e| AppError::TextInjectionFailed {
        reason: e.to_string(),
    })?;

    // enigo 0.6's `Settings` does NOT expose a cross-platform inter-keystroke
    // delay field (the historical `linux_delay` was removed). The default
    // `Keyboard::text` therefore injects the whole string back-to-back. To
    // honour the configured `injection.delay_ms` (spec 14 §2.3) we replay the
    // same per-character loop the default `text()` uses, but sleep between
    // characters. When delay_ms is 0 the fast path is identical to `text()`.
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
        enigo.key(Key::Unicode(c), Direction::Click).map_err(|e| {
            AppError::TextInjectionFailed {
                reason: e.to_string(),
            }
        })?;
        sleep(per_char);
    }

    Ok(())
}

/// Hand-written mock [`Injector`] for tests (spec 06 §2.3 — minimal mock; the
/// project does not depend on `mockall`).
///
/// Records every `inject` call's `(text, delay_ms)` so tests can assert that
/// the queue processor dispatched the expected payloads without touching the
/// OS input stack. Validation still runs first, so passing oversized/empty
/// text surfaces the same `AppError` as the production path.
#[cfg(test)]
#[derive(Default)]
pub struct MockInjector {
    pub calls: Mutex<Vec<(String, u64)>>,
}

#[cfg(test)]
impl MockInjector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of recorded calls (text, delay_ms) in insertion order.
    pub fn recorded(&self) -> Vec<(String, u64)> {
        self.calls.lock().expect("mock calls lock poisoned").clone()
    }
}

#[cfg(test)]
impl Injector for MockInjector {
    fn inject(&self, text: &str, delay_ms: u64) -> AppResult<()> {
        validate_text(text)?;
        self.calls
            .lock()
            .expect("mock calls lock poisoned")
            .push((text.to_string(), delay_ms));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_text_rejects_empty() {
        let err = validate_text("").unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
    }

    #[test]
    fn validate_text_rejects_too_long() {
        let long = "a".repeat(MAX_TEXT_LENGTH + 1);
        let err = validate_text(&long).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_TOO_LONG");
        assert_eq!(
            err.error_context(),
            serde_json::json!({ "length": MAX_TEXT_LENGTH + 1, "max": MAX_TEXT_LENGTH })
        );
    }

    #[test]
    fn validate_text_accepts_exactly_max_length() {
        // Pure validation — no side effect, no enigo. This replaces the former
        // test that called inject_text with 10000 'a's and let enigo actually
        // type them into the focused window.
        let exact = "a".repeat(MAX_TEXT_LENGTH);
        validate_text(&exact).unwrap();
    }

    #[test]
    fn mock_injector_records_calls_and_runs_validation() {
        let mock = MockInjector::new();
        mock.inject("hello", 10).unwrap();
        mock.inject("world", 0).unwrap();
        assert_eq!(
            mock.recorded(),
            vec![("hello".to_string(), 10), ("world".to_string(), 0)]
        );
    }

    #[test]
    fn mock_injector_surfaces_validation_errors() {
        let mock = MockInjector::new();
        let err = mock.inject("", 0).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
        // Validation failure must not record a call.
        assert!(mock.recorded().is_empty());
    }

    #[test]
    fn enigo_injector_validates_before_delegating() {
        // EnigoInjector's validation path is exercised without depending on a
        // display server: empty/oversized input returns before enigo is built.
        let inj = EnigoInjector::new();
        let err = inj.inject("", 0).unwrap_err();
        assert_eq!(err.error_code(), "TEXT_EMPTY");
        let long = "a".repeat(MAX_TEXT_LENGTH + 1);
        let err = inj.inject(&long, 0).unwrap_err();
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
    fn max_text_length_is_ten_thousand() {
        assert_eq!(MAX_TEXT_LENGTH, 10_000);
    }
}

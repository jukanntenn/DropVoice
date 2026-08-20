//! Integration tests for the text injection module.
//!
//! These tests exercise the *validation* boundary (`validate_text`) and the
//! `Injector` trait wiring — they intentionally never touch the OS input
//! stack. The historical version called `inject_text` with 10 000 'a's and
//! relied on enigo actually typing them into the focused window, which
//! hijacked the developer's keyboard during `cargo test`. enigo provides no
//! mock of its own ("we assume it worked as long as there was no panic",
//! `enigo/src/tests/keyboard.rs`), so the mockable boundary lives in our
//! `Injector` trait (spec 06 §2.3 — mock boundary dependencies).
//!
//! Note: integration tests live in a separate crate and cannot see the
//! `#[cfg(test)]` `MockInjector` defined inside the lib. The lib's own unit
//! tests cover the mock-wired dispatch path; here we only assert the
//! side-effect-free validation path and the public type contracts.

use dropvoice_desktop_lib::error::AppError;
use dropvoice_desktop_lib::text::{
    validate_text, EnigoInjector, Injector, DEFAULT_MAX_TEXT_LENGTH,
};

#[test]
fn empty_text_returns_text_empty_error() {
    let err = validate_text("", DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
    assert_eq!(err.error_code(), "TEXT_EMPTY");
}

#[test]
fn text_over_max_length_returns_text_too_long() {
    let long = "a".repeat(DEFAULT_MAX_TEXT_LENGTH + 1);
    let err = validate_text(&long, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
    assert_eq!(err.error_code(), "TEXT_TOO_LONG");
    let ctx = err.error_context();
    assert_eq!(ctx["length"], DEFAULT_MAX_TEXT_LENGTH + 1);
    assert_eq!(ctx["max"], DEFAULT_MAX_TEXT_LENGTH);
}

#[test]
fn text_at_max_length_passes_validation() {
    // Pure validation — no enigo, no OS input. This replaces the former test
    // that let enigo type 10 000 'a's into whatever window had focus.
    let exact = "a".repeat(DEFAULT_MAX_TEXT_LENGTH);
    validate_text(&exact, DEFAULT_MAX_TEXT_LENGTH).unwrap();
}

#[test]
fn enigo_injector_validates_before_injection() {
    // EnigoInjector must run the same validation before delegating to enigo,
    // so empty/oversized input is rejected without depending on a display
    // server. The success path (real enigo call) is intentionally not
    // exercised here — that is the lib unit tests' mock-wired responsibility.
    let inj = EnigoInjector::new();
    let err = inj.inject("", 0, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
    assert_eq!(err.error_code(), "TEXT_EMPTY");

    let long = "a".repeat(DEFAULT_MAX_TEXT_LENGTH + 1);
    let err = inj.inject(&long, 0, DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
    assert_eq!(err.error_code(), "TEXT_TOO_LONG");
}

#[test]
fn validate_text_error_serialises_as_code_message_context() {
    let err = validate_text("", DEFAULT_MAX_TEXT_LENGTH).unwrap_err();
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["code"], "TEXT_EMPTY");
    assert!(json["message"].as_str().unwrap().contains("empty"));
}

#[test]
fn app_error_implements_send_sync() {
    // Compile-time check that AppError is Send + Sync (required for
    // returning from Tauri commands on a different thread).
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<AppError>();
    // The Injector trait object must also be Send + Sync so it can be held
    // behind Arc<dyn Injector> across the tokio runtime.
    assert_send_sync::<dyn Injector>();
}

#[test]
fn default_max_text_length_is_ten_thousand() {
    assert_eq!(DEFAULT_MAX_TEXT_LENGTH, 10_000);
}

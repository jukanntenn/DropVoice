pub mod injector;

pub use injector::{validate_text, EnigoInjector, Injector, DEFAULT_MAX_TEXT_LENGTH};

#[cfg(test)]
pub use injector::MockInjector;

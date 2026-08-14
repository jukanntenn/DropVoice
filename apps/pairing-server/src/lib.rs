//! 库入口，供集成测试与 main.rs 共用。

pub mod api;
pub mod batch;
pub mod cache;
pub mod clock;
pub mod config;
pub mod docs;
pub mod domain;
pub mod error;
pub mod observability;
pub mod store;

#[cfg(test)]
pub mod test_support;

pub use api::state::AppState;

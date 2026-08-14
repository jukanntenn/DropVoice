//! 时钟抽象（可测性）。生产用 SystemClock，测试用 FakeClock。
//!
//! spec 21 §3.1 Pre-req-1：所有时间相关逻辑通过 Clock trait 获取"当前时间"，
//! 测试时注入 FakeClock 实现确定性过期/续期判定。

use chrono::{DateTime, Utc};

/// 时钟 trait，供 repo/handler 层获取当前时间。
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// 生产时钟，包装 chrono::Utc::now()。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// 测试时钟，时间可控。
#[derive(Debug, Clone)]
pub struct FakeClock {
    inner: std::sync::Arc<std::sync::Mutex<DateTime<Utc>>>,
}

impl FakeClock {
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(initial)),
        }
    }

    pub fn advance(&self, dur: chrono::Duration) {
        let mut g = self.inner.lock().unwrap();
        *g += dur;
    }

    pub fn set(&self, t: DateTime<Utc>) {
        *self.inner.lock().unwrap() = t;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.inner.lock().unwrap()
    }
}

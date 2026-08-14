//! Token 缓存：`token -> device_id`，认证时优先走缓存。
//!
//! 大小限制：Token 100 万条。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

const MAX_TOKENS: usize = 1_000_000;

/// Token 缓存。
#[derive(Debug, Clone)]
pub struct Cache {
    tokens: Arc<RwLock<HashMap<String, Uuid>>>,
    max_tokens: usize,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            max_tokens: MAX_TOKENS,
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    /// 测试用构造器：可自定义上限。
    #[cfg(test)]
    pub fn with_limits(max_tokens: usize) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            max_tokens,
        }
    }

    pub async fn put_token(&self, token: &str, device_id: Uuid) {
        let mut map = self.tokens.write().await;
        if map.len() >= self.max_tokens {
            // 简单驱逐：移除一个任意项（Token 缓存过期由 DB 过期驱动，不严格 LRU）。
            if let Some(k) = map.keys().next().cloned() {
                map.remove(&k);
            }
        }
        map.insert(token.to_string(), device_id);
    }

    pub async fn get_token(&self, token: &str) -> Option<Uuid> {
        self.tokens.read().await.get(token).copied()
    }

    pub async fn invalidate_token(&self, token: &str) {
        self.tokens.write().await.remove(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UT-cache-01：token 缓存续期失效。
    #[tokio::test]
    async fn token_cache_invalidate_old_on_renew() {
        let cache = Cache::new();
        let id = Uuid::new_v4();

        cache.put_token("old-token", id).await;
        assert_eq!(cache.get_token("old-token").await, Some(id));

        // 续期：失效旧 token，写入新 token。
        cache.invalidate_token("old-token").await;
        cache.put_token("new-token", id).await;

        assert!(cache.get_token("old-token").await.is_none());
        assert_eq!(cache.get_token("new-token").await, Some(id));
    }

    /// UT-cache-02：invalidate_token。
    #[tokio::test]
    async fn invalidate_token_works() {
        let cache = Cache::new();
        let id = Uuid::new_v4();
        cache.put_token("my-token", id).await;

        assert_eq!(cache.get_token("my-token").await, Some(id));
        cache.invalidate_token("my-token").await;
        assert!(cache.get_token("my-token").await.is_none());
    }

    /// UT-cache-03：token 缓存驱逐（用小上限）。
    #[tokio::test]
    async fn token_cache_eviction() {
        let cache = Cache::with_limits(2);
        let id = Uuid::new_v4();

        cache.put_token("a", id).await;
        cache.put_token("b", id).await;
        cache.put_token("c", id).await; // 触发驱逐

        // "c" 一定在（最后插入），"a" 或 "b" 中有一个被驱逐。
        assert!(cache.get_token("c").await.is_some());
        let remaining = cache.get_token("a").await.is_some() as usize
            + cache.get_token("b").await.is_some() as usize;
        assert_eq!(remaining, 1);
    }

    /// UT-cache-04：token 基本生命周期。
    #[tokio::test]
    async fn token_cache_basic_lifecycle() {
        let cache = Cache::new();
        let id = Uuid::new_v4();

        cache.put_token("my-token", id).await;
        assert_eq!(cache.get_token("my-token").await, Some(id));

        cache.invalidate_token("my-token").await;
        assert!(cache.get_token("my-token").await.is_none());
    }

    /// UT-cache-05：未命中返回 None。
    #[tokio::test]
    async fn token_cache_miss() {
        let cache = Cache::new();
        assert!(cache.get_token("nonexistent").await.is_none());
    }
}

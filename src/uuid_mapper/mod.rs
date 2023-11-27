use std::sync::Arc;

use async_trait::async_trait;

use crate::uuid_mapper::in_memory::InMemoryUuidMapper;
#[cfg(feature = "redis-storage")]
use crate::uuid_mapper::redis::RedisUuidMapper;

mod in_memory;
#[cfg(feature = "redis-storage")]
mod redis;

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Vec<String>;
    async fn get(&self, bot_uuid: &str) -> Option<V>;
}

#[cfg(feature = "redis-storage")]
const REDIS_URL_ENV: &str = "REDIS_URL";

pub fn create_arc<V: Clone + Sync + Send + 'static>() -> Arc<dyn UuidMapper<V>> {
    #[cfg(feature = "redis-storage")]
    if let Ok(redis_url) = std::env::var(REDIS_URL_ENV) {
        return Arc::new(RedisUuidMapper::new(&redis_url)
            .unwrap_or_else(|e| panic!("Cannot create Redis client from {REDIS_URL_ENV}=\"{redis_url}\": {e}")))
    };
    Arc::new(InMemoryUuidMapper::new())
}

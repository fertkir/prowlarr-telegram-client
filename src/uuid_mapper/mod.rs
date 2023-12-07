use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::uuid_mapper::in_memory::InMemoryUuidMapper;
#[cfg(feature = "redis-storage")]
use crate::uuid_mapper::redis::RedisUuidMapper;

mod in_memory;
#[cfg(feature = "redis-storage")]
mod redis;

#[derive(Debug)]
pub enum MapperError { // todo use thiserror: https://docs.rs/thiserror/latest/thiserror/
    Err(String)
}

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> where V: 'async_trait;
    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError>;
}

#[cfg(feature = "redis-storage")]
const REDIS_URL_ENV: &str = "REDIS_URL";

pub fn create<V: Clone + Sync + Send + Serialize + DeserializeOwned + 'static>() -> Box<dyn UuidMapper<V>> {
    #[cfg(feature = "redis-storage")]
    if let Ok(redis_url) = std::env::var(REDIS_URL_ENV) {
        return Box::new(RedisUuidMapper::new(&redis_url)
            .unwrap_or_else(|e| panic!("Cannot create Redis client from {REDIS_URL_ENV}=\"{redis_url}\": {e}")))
    };
    Box::new(InMemoryUuidMapper::new())
}

use crate::core::traits::uuid_mapper::UuidMapper;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::ext::uuid_mapper::in_memory::InMemoryUuidMapper;
#[cfg(feature = "redis-storage")]
use crate::ext::uuid_mapper::redis::RedisUuidMapper;

mod in_memory;
#[cfg(feature = "redis-storage")]
mod redis;

#[cfg(feature = "redis-storage")]
const REDIS_URL_ENV: &str = "REDIS_URL";

pub fn create<
    #[cfg(feature = "redis-storage")] V: Clone + Sync + Send + Serialize + DeserializeOwned + 'static,
    #[cfg(not(feature = "redis-storage"))] V: Clone + Sync + Send + 'static,
>() -> Box<dyn UuidMapper<V>> {
    #[cfg(feature = "redis-storage")]
    if let Ok(redis_url) = std::env::var(REDIS_URL_ENV) {
        return Box::new(RedisUuidMapper::new(&redis_url)
            .unwrap_or_else(|e| panic!("Cannot create Redis client from {REDIS_URL_ENV}=\"{redis_url}\": {e}")))
    };
    Box::new(InMemoryUuidMapper::new())
}

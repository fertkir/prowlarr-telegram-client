use std::sync::Arc;

use async_trait::async_trait;

use crate::uuid_mapper::in_memory::InMemoryUuidMapper;

mod in_memory;

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Vec<String>;
    async fn get(&self, bot_uuid: &str) -> Option<V>;
}

pub fn create_arc<V: Clone + Sync + Send + 'static>() -> Arc<dyn UuidMapper<V>> {
    Arc::new(InMemoryUuidMapper::new())
}

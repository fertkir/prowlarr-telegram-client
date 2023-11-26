use async_trait::async_trait;

pub mod in_memory;

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Vec<String>;
    async fn get(&self, bot_uuid: &str) -> Option<V>;
}

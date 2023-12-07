use async_trait::async_trait;

#[derive(Debug)]
pub enum MapperError {
    // todo use thiserror: https://docs.rs/thiserror/latest/thiserror/
    Err(String)
}

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> where V: 'async_trait;
    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError>;
}

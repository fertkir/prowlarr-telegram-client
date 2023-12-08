use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapperError {
    #[error("Error when interacting with mapper: {0}")]
    Err(String)
}

#[async_trait]
pub trait UuidMapper<V>: Sync + Send {
    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> where V: 'async_trait;
    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError>;
}

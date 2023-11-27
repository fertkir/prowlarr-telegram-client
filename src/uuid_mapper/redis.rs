use async_trait::async_trait;

use crate::uuid_mapper::UuidMapper;

pub struct RedisUuidMapper<V> {
    value: Vec<V>,
    client: redis::Client
}

impl<V> RedisUuidMapper<V> {

    pub fn new(url: &str) -> Result<RedisUuidMapper<V>, String> {
        Ok(RedisUuidMapper {
            value: Vec::new(),
            client: redis::Client::open(url).map_err(|e|e.to_string())?
        })
    }
}

#[async_trait]
impl<V: Sync + Send> UuidMapper<V> for RedisUuidMapper<V> {
    async fn put_all(&self, values: Vec<V>) -> Vec<String> {
        todo!()
    }

    async fn get(&self, bot_uuid: &str) -> Option<V> {
        todo!()
    }
}
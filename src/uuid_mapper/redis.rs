use async_trait::async_trait;
use redis::{AsyncCommands, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error;

use crate::uuid_mapper::{MapperError, UuidMapper};

pub struct RedisUuidMapper {
    client: redis::Client
}

impl RedisUuidMapper {

    pub fn new(url: &str) -> Result<RedisUuidMapper, String> {
        Ok(RedisUuidMapper {
            client: redis::Client::open(url).map_err(|e|e.to_string())?
        })
    }
}

#[async_trait]
impl<V: Serialize + Sync + Send + DeserializeOwned> UuidMapper<V> for RedisUuidMapper {
    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> where V: 'async_trait {
        let serialized_values: Vec<String> = values.iter()
            .map(|value| serde_json::to_string(value).unwrap())  // todo no unwrap
            .collect();
        let mut con = self.client.get_async_connection().await?;
        let seq: usize = con.incr("uuid-mapper:sequence", values.len()).await?; // todo do not start with 1
        let offset = seq - values.len();
        let x: Vec<(String, String)> = serialized_values.into_iter().enumerate()
            .map(|(index, value)| (format!("uuid-mapper:uuid:{}", (offset + index)), value))
            .collect();
        con.mset(&x).await?; // todo set expiration: https://github.com/redis/ioredis/issues/1133#issuecomment-630351474
        Ok((offset..(values.len() + offset)).map(|a|a.to_string()).collect())
    }

    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError> {
        let mut con = self.client.get_async_connection().await?;
        let x: Option<String> = con.get(format!("uuid-mapper:uuid:{}", bot_uuid)).await?;
        match x {
            None => Ok(None),
            Some(x) => {
                let option = serde_json::from_str(&x)?;
                Ok(option)
            }
        }
    }
}

impl From<RedisError> for MapperError {
    fn from(value: RedisError) -> Self {
        MapperError::Err(value.to_string())
    }
}

impl From<Error> for MapperError {
    fn from(value: Error) -> Self {
        MapperError::Err(value.to_string())
    }
}

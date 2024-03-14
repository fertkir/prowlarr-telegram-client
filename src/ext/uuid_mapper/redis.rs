use async_trait::async_trait;
use redis::{AsyncCommands, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error;

use crate::core::traits::uuid_mapper::MapperError;
use crate::uuid_mapper::UuidMapper;

pub struct RedisUuidMapper {
    client: redis::Client,
    sequence_start: usize,
    key_expiration: u64
}

const REDIS_SEQUENCE_START_ENV: &str = "REDIS_SEQUENCE_START";
const REDIS_KEY_EXPIRATION_ENV: &str = "REDIS_KEY_EXPIRATION";
const ONE_WEEK: &str = "604800";
const DEFAULT_SEQUENCE_START: &str = "1000";
const SEQUENCE_KEY: &str = "uuid-mapper:sequence";
const UUID_KEY_PREFIX: &str = "uuid-mapper:uuid";

impl RedisUuidMapper {

    pub fn new(url: &str) -> Result<RedisUuidMapper, String> {
        Ok(RedisUuidMapper {
            client: redis::Client::open(url)
                .map_err(|e|e.to_string())?,
            sequence_start: std::env::var(REDIS_SEQUENCE_START_ENV)
                .unwrap_or_else(|_| DEFAULT_SEQUENCE_START.to_string())
                .parse()
                .unwrap_or_else(|_| panic!("{REDIS_SEQUENCE_START_ENV} must be integer")),
            key_expiration: std::env::var(REDIS_KEY_EXPIRATION_ENV)
                .unwrap_or_else(|_| ONE_WEEK.to_string())
                .parse()
                .unwrap_or_else(|_| panic!("{REDIS_KEY_EXPIRATION_ENV} must be integer"))
        })
    }
}


#[async_trait]
impl<V: Serialize + Sync + Send + DeserializeOwned> UuidMapper<V> for RedisUuidMapper {
    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> where V: 'async_trait {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let seq: Vec<usize> = redis::pipe().atomic()
            .set_nx(SEQUENCE_KEY, self.sequence_start.to_string()).ignore()
            .incr(SEQUENCE_KEY, values.len())
            .query_async(&mut con).await?;
        let offset = seq[0] - values.len();
        let mut pipe = redis::pipe();
        pipe.atomic();
        for (index, value) in values.iter().enumerate() {
            let key = format!("{}:{}", UUID_KEY_PREFIX, offset + index);
            let value = serde_json::to_string(value)?;
            pipe.set_ex(key, value, self.key_expiration).ignore();
        }
        pipe.query_async(&mut con).await?;
        Ok((offset..(values.len() + offset)).map(|a|a.to_string()).collect())
    }

    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let x: Option<String> = con.get(format!("{}:{}", UUID_KEY_PREFIX, bot_uuid)).await?;
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

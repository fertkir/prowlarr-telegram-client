use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use dashmap::DashMap;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::core::ext::uuid_mapper::{MapperError, UuidMapper};

pub struct InMemoryUuidMapper<V: Clone> {
    session_key: String,
    map: DashMap<String, V>,
    sequence: AtomicU32
}

const UUID_RANDOM_PART_LENGTH: usize = 6;
const MAX_CACHE_SIZE: u32 = 10_000;

impl<V: Clone> InMemoryUuidMapper<V> {
    pub fn new() -> InMemoryUuidMapper<V> {
        InMemoryUuidMapper {
            session_key: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(UUID_RANDOM_PART_LENGTH)
                .map(char::from)
                .collect::<String>(),
            map: DashMap::new(),
            sequence: AtomicU32::new(1)
        }
    }

    fn put(&self, value: V) -> String {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let bot_uuid = format!("{}{}", self.session_key, seq);
        if seq > MAX_CACHE_SIZE {
            self.map.remove(&format!("{}{}", self.session_key, seq - MAX_CACHE_SIZE));
        }
        self.map.insert(bot_uuid.clone(), value);
        bot_uuid
    }
}

#[async_trait]
impl<V: Clone + Sync + Send> UuidMapper<V> for InMemoryUuidMapper<V> {

    async fn put_all(&self, values: Vec<V>) -> Result<Vec<String>, MapperError> {
        Ok(values.into_iter()
            .map(|v| self.put(v))
            .collect())
    }

    async fn get(&self, bot_uuid: &str) -> Result<Option<V>, MapperError> {
        Ok(self.map.get(bot_uuid).map(|e| e.value().clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::uuid_mapper::in_memory::InMemoryUuidMapper;
    use crate::uuid_mapper::UuidMapper;

    #[tokio::test]
    async fn get_same_value_multiple_times() {
        let mapper = InMemoryUuidMapper::<String>::new();
        let keys = mapper.put_all(vec!["value 1".to_string()]).await.unwrap();

        assert_eq!(mapper.get(&keys[0]).await.unwrap(), Some("value 1".to_string()));
        assert_eq!(mapper.get(&keys[0]).await.unwrap(), Some("value 1".to_string()));
    }

    #[tokio::test]
    async fn put_and_get_different_values() {
        let mapper = InMemoryUuidMapper::<String>::new();
        let keys = mapper.put_all(vec!["value 1".to_string(), "value 2".to_string()]).await.unwrap();

        assert_eq!(mapper.get(&keys[0]).await.unwrap(), Some("value 1".to_string()));
        assert_eq!(mapper.get(&keys[1]).await.unwrap(), Some("value 2".to_string()));
    }

    #[tokio::test]
    async fn get_none_if_key_is_unknown() {
        let mapper = InMemoryUuidMapper::<String>::new();

        assert_eq!(mapper.get("key1").await.unwrap(), None);
    }
}

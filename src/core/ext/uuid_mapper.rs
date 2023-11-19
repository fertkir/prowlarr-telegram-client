use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct UuidMapper<V: Clone> {
    session_key: String,
    map: DashMap<String, V>,
    sequence: AtomicU32
}

const UUID_RANDOM_PART_LENGTH: usize = 6;
const MAX_CACHE_SIZE: u32 = 10_000;

impl<V: Clone> UuidMapper<V> {
    pub fn new() -> UuidMapper<V> {
        UuidMapper {
            session_key: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(UUID_RANDOM_PART_LENGTH)
                .map(char::from)
                .collect::<String>(),
            map: DashMap::new(),
            sequence: AtomicU32::new(1)
        }
    }

    pub fn put(&self, value: V) -> String {
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        let bot_uuid = format!("{}{}", self.session_key, seq);
        if seq > MAX_CACHE_SIZE {
            self.map.remove(&format!("{}{}", self.session_key, seq - MAX_CACHE_SIZE));
        }
        self.map.insert(bot_uuid.clone(), value);
        bot_uuid
    }

    pub fn get(&self, bot_uuid: &str) -> Option<V> {
        self.map.get(bot_uuid).map(|e| e.value().clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ext::uuid_mapper::UuidMapper;

    #[test]
    fn get_same_value_multiple_times() {
        let mapper = UuidMapper::<String>::new();
        let key1 = mapper.put("value 1".to_string());

        assert_eq!(mapper.get(&key1), Some("value 1".to_string()));
        assert_eq!(mapper.get(&key1), Some("value 1".to_string()));
    }

    #[test]
    fn put_and_get_different_values() {
        let mapper = UuidMapper::<String>::new();
        let key1 = mapper.put("value 1".to_string());
        let key2 = mapper.put("value 2".to_string());

        assert_eq!(mapper.get(&key1), Some("value 1".to_string()));
        assert_eq!(mapper.get(&key2), Some("value 2".to_string()));
    }

    #[test]
    fn get_none_if_key_is_unknown() {
        let mapper = UuidMapper::<String>::new();

        assert_eq!(mapper.get("key1"), None);
    }
}

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
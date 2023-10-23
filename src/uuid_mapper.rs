use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct UuidMapper<V: Clone> {
    session_key: String,
    map: DashMap<String, V>,
    sequence: AtomicU32
}

impl<V: Clone> UuidMapper<V> {
    pub fn new() -> UuidMapper<V> {
        UuidMapper {
            session_key: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect::<String>(),
            map: DashMap::new(),
            sequence: AtomicU32::new(1)
        }
    }

    pub fn put(&self, value: V) -> String { // todo what's the difference between self and &self?
        let bot_uuid = format!(
            "{}{}", self.session_key,  self.sequence.fetch_add(1, Ordering::SeqCst));
        self.map.insert(bot_uuid.clone(), value);
        bot_uuid
        // todo remove old keys to prevent memory leaks
    }

    pub fn get(&self, bot_uuid: &str) -> Option<V> {
        self.map.get(bot_uuid).map(|e| e.value().clone())
    }
}
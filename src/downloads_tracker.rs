use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use teloxide::types::ChatId;

#[derive(Eq)]
pub struct User {
    pub chat_id: ChatId,
    pub locale: String
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.chat_id == other.chat_id
    }
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.chat_id.hash(state)
    }
}

pub struct DownloadsTracker {
    users_by_download: Mutex<HashMap<String, HashSet<User>>> // todo is there a normal lock-free HashMap in rust?
}

impl DownloadsTracker {

    pub fn new() -> DownloadsTracker {
        DownloadsTracker {
            users_by_download: Mutex::new(HashMap::new())
        }
    }

    pub fn add(&self, hash: String, chat_id: ChatId, locale: String) {
        let mut users_by_download = self.users_by_download.lock().unwrap();
        users_by_download.entry(hash)
            .and_modify(|users| {
                users.insert(User { chat_id, locale: locale.clone()});
            })
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(User { chat_id, locale: locale.clone() });
                set
            });
    }

    pub fn remove(&self, hash: String) -> HashSet<User> {
        let mut users_by_download = self.users_by_download.lock().unwrap();
        users_by_download.remove(&hash).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use teloxide::types::ChatId;

    use crate::downloads_tracker::{DownloadsTracker, User};

    #[test]
    fn add_and_remove() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), ChatId(1), "en".to_string());
        tracker.add("hash1".to_string(), ChatId(2), "ru".to_string());
        tracker.add("hash2".to_string(), ChatId(2), "ru".to_string());

        let hash1_users = tracker.remove("hash1".to_string());
        assert_eq!(hash1_users.len(), 2);
        assert_eq!(hash1_users.contains(&User { chat_id: ChatId(1), locale: "en".to_string() }), true);
        assert_eq!(hash1_users.contains(&User { chat_id: ChatId(2), locale: "ru".to_string() }), true);

        let hash2_users = tracker.remove("hash2".to_string());
        assert_eq!(hash2_users.len(), 1);
        assert_eq!(hash2_users.contains(&User { chat_id: ChatId(2), locale: "ru".to_string() }), true);

        let hash3_users = tracker.remove("hash3".to_string());
        assert_eq!(hash3_users.len(), 0);
    }
}

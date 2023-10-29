use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use teloxide::types::ChatId;

#[derive(Eq, Hash, PartialEq)]
pub struct User {
    pub chat_id: ChatId,
    pub locale: String
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

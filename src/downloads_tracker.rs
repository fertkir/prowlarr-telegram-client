use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use teloxide::types::ChatId;

pub struct DownloadsTracker {
    users_by_download: Mutex<HashMap<String, HashSet<ChatId>>> // todo is there a normal lock-free HashMap in rust?
}

impl DownloadsTracker {

    pub fn new() -> DownloadsTracker {
        DownloadsTracker {
            users_by_download: Mutex::new(HashMap::new())
        }
    }

    pub fn add(&self, hash: String, user: ChatId) {
        let mut users_by_download = self.users_by_download.lock().unwrap();
        users_by_download.entry(hash)
            .and_modify(|users| {
                users.insert(user);
            })
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(user);
                set
            });
    }

    pub fn remove(&self, hash: String) -> HashSet<ChatId> {
        let mut users_by_download = self.users_by_download.lock().unwrap();
        users_by_download.remove(&hash).unwrap_or_default()
    }
}

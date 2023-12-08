use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use dashmap::DashMap;

use crate::core::traits::input::{Destination, Locale};

#[derive(Eq)]
pub struct User {
    pub destination: Destination,
    pub locale: Locale
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.destination.hash(state)
    }
}

pub struct DownloadsTracker {
    users_by_download: DashMap<String, HashSet<User>>
}

impl DownloadsTracker {

    pub fn new() -> DownloadsTracker {
        DownloadsTracker {
            users_by_download: DashMap::new()
        }
    }

    pub fn add(&self, hash: String, destination: Destination, locale: Locale) {
        // this entry() call should keep a lock during returned value's lifetime:
        // https://github.com/xacrimon/dashmap/issues/78#issuecomment-633745091
        self.users_by_download.entry(hash)
            .or_default()
            .insert(User { destination, locale });
    }

    pub fn remove(&self, hash: String) -> HashSet<User> {
        self.users_by_download.remove(&hash)
            .map(|entry| entry.1)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::downloads_tracker::{DownloadsTracker, User};

    #[test]
    fn one_user_for_one_hash() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 2, "ru".into());

        let hash2_users = tracker.remove("hash1".to_string());
        assert_eq!(hash2_users.len(), 1);
        assert!(hash2_users.contains(&User { destination: 2, locale: "ru".into() }));
    }

    #[test]
    fn return_empty_set_if_unknown_hash() {
        let tracker = DownloadsTracker::new();

        let hash3_users = tracker.remove("hash3".to_string());
        assert_eq!(hash3_users.len(), 0);
    }

    #[test]
    fn multiple_users_for_same_hash() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 1, "en".into());
        tracker.add("hash1".to_string(), 2, "ru".into());

        let hash1_users = tracker.remove("hash1".to_string());
        assert_eq!(hash1_users.len(), 2);
        assert!(hash1_users.contains(&User { destination: 1, locale: "en".into() }));
        assert!(hash1_users.contains(&User { destination: 2, locale: "ru".into() }));
    }

    #[test]
    fn user_with_same_id_is_same_user() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 1, "ru".into());
        tracker.add("hash1".to_string(), 1, "en".into());

        let hash1_users = tracker.remove("hash1".to_string());
        assert_eq!(hash1_users.len(), 1);
        assert!(hash1_users.contains(&User { destination: 1, locale: "en".into() }));
    }

    #[test]
    fn remove_method_should_remove_value_from_tracker() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 1, "ru".into());

        assert_eq!(tracker.remove("hash1".to_string()).len(), 1);
        assert_eq!(tracker.remove("hash1".to_string()).len(), 0);
    }
}

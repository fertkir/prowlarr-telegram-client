use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use dashmap::DashMap;

use crate::core::ext::input::Destination;

#[derive(Eq)]
pub struct User {
    pub destination: Destination,
    pub locale: String
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

    pub fn add(&self, hash: String, destination: Destination, locale: String) {
        // this entry() call should keep a lock during returned value's lifetime:
        // https://github.com/xacrimon/dashmap/issues/78#issuecomment-633745091
        self.users_by_download.entry(hash)
            .or_default()
            .insert(User { destination, locale: locale.clone()});
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
        tracker.add("hash1".to_string(), 2, "ru".to_string());

        let hash2_users = tracker.remove("hash1".to_string());
        assert_eq!(hash2_users.len(), 1);
        assert!(hash2_users.contains(&User { destination: 2, locale: "ru".to_string() }));
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
        tracker.add("hash1".to_string(), 1, "en".to_string());
        tracker.add("hash1".to_string(), 2, "ru".to_string());

        let hash1_users = tracker.remove("hash1".to_string());
        assert_eq!(hash1_users.len(), 2);
        assert!(hash1_users.contains(&User { destination: 1, locale: "en".to_string() }));
        assert!(hash1_users.contains(&User { destination: 2, locale: "ru".to_string() }));
    }

    #[test]
    fn user_with_same_id_is_same_user() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 1, "ru".to_string());
        tracker.add("hash1".to_string(), 1, "en".to_string());

        let hash1_users = tracker.remove("hash1".to_string());
        assert_eq!(hash1_users.len(), 1);
        assert!(hash1_users.contains(&User { destination: 1, locale: "en".to_string() }));
    }

    #[test]
    fn remove_method_should_remove_value_from_tracker() {
        let tracker = DownloadsTracker::new();
        tracker.add("hash1".to_string(), 1, "ru".to_string());

        assert_eq!(tracker.remove("hash1".to_string()).len(), 1);
        assert_eq!(tracker.remove("hash1".to_string()).len(), 0);
    }
}

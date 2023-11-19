pub type UserId = u64;

pub struct PermissionChecker {
    pub allowed_users: Vec<UserId>
}

impl PermissionChecker {
    pub fn is_allowed(&self, user_id: &UserId) -> bool {
        self.allowed_users.is_empty() || self.allowed_users.contains(user_id)
    }
}

use super::email::Email;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod display_user;
pub mod user_config;
mod user_id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub email: Email,
    pub display_name: Option<String>,
    pub id: UserId,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserId {
    id: Uuid,
}
impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

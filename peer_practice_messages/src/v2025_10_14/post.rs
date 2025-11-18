use super::level::Level;
use super::user::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
pub use topics::Topics;
use uuid::Uuid;

mod topics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub title: Topics,
    pub content: String,
    pub level: Level,
    pub owner: UserId,
    pub date: DateTime<Utc>,
    pub partaking_users: HashSet<UserId>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostId {
    id: Uuid,
}

impl std::fmt::Display for PostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Default for PostId {
    fn default() -> Self {
        Self::new()
    }
}

impl PostId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub const NULL: Self = Self { id: Uuid::nil() };
}

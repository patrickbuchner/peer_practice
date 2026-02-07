use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId {
    id: Uuid,
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn from_id(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct SessionInformation {
    pub session_id: SessionId,
    pub description: String,
}

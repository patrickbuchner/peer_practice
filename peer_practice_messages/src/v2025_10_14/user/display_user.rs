use super::super::user::{User, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDisplay {
    pub display_name: Option<String>,
    pub id: UserId,
}

impl From<User> for UserDisplay {
    fn from(user: User) -> Self {
        UserDisplay {
            display_name: user.display_name,
            id: user.id,
        }
    }
}

impl From<&User> for UserDisplay {
    fn from(user: &User) -> Self {
        UserDisplay {
            display_name: user.display_name.clone(),
            id: user.id,
        }
    }
}

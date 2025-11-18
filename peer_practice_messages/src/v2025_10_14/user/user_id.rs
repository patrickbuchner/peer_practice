use super::super::user::UserId;
use uuid::Uuid;

impl UserId {
    pub fn test() -> Self {
        UserId {
            id: Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap(),
        }
    }
    pub fn new() -> Self {
        UserId { id: Uuid::new_v4() }
    }
}

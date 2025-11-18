use peer_practice_shared::user::UserId;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: UserId,
    pub exp: usize,
}

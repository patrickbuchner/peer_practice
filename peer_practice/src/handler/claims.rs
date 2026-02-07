use peer_practice_messages::current::sessions::SessionId;
use peer_practice_messages::current::user::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: UserId,
    pub exp: usize,
    pub client_id: Option<SessionId>,
}

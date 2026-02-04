use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::active_sessions::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: UserId,
    pub exp: usize,
    pub client_id: Option<ClientId>,
}

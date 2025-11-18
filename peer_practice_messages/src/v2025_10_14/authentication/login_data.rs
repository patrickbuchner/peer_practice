use super::super::authentication::method::AuthenticationMethod;
use super::super::email::Email;
use super::super::user::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub email: Email,
    pub auth: AuthenticationMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinLogin {
    pub pin: String,
    pub id: UserId,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    EmailOTP,
    Password(String),
}

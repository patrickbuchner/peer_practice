use serde::{Deserialize, Serialize};
use email::EmailConfig;
use server::ServerConfig;

pub mod envelope;
pub mod email;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub email: EmailConfig,
    pub server: ServerConfig,
}

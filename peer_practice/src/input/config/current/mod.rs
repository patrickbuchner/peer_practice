use crate::input::config::current::email::EmailConfig;
use crate::input::config::current::server::ServerConfig;
use serde::{Deserialize, Serialize};
pub type Envelope = crate::input::config::v2025_11_23::envelope::V2025_11_23Config;

pub mod email;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub email: EmailConfig,
    pub server: ServerConfig,
}

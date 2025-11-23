pub(crate) use email::EmailConfig;
use serde::{Deserialize, Serialize};
use server::ServerConfig;

pub mod email;
pub mod envelope;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub email: EmailConfig,
    pub server: ServerConfig,
}

impl From<Config> for crate::input::config::current::Config {
    fn from(value: Config) -> Self {
        Self {
            email: value.email.into(),
            server: value.server.into(),
        }
    }
}

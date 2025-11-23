use email::EmailConfig;
use serde::{Deserialize, Serialize};
use server::ServerConfig;

pub mod email;
pub mod envelope;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub email: EmailConfig,
    pub server: ServerConfig,
}

impl TryFrom<Config> for crate::input::config::current::Config {
    type Error = eyre::Error;
    fn try_from(value: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            email: value.email.try_into()?,
            server: value.server.try_into()?,
        })
    }
}

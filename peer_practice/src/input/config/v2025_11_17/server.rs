use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub webroot: Option<PathBuf>,
    pub jwt_secret: String,
    pub data_dir: PathBuf,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
}

impl From<ServerConfig> for crate::input::config::current::server::ServerConfig {
    fn from(value: ServerConfig) -> Self {
        Self {
            jwt_secret: value.jwt_secret,
            data_dir: value.data_dir,
            port: value.port,
            webroot: value.webroot,
            cors_allowed_origins: value.cors_allowed_origins,
        }
    }
}

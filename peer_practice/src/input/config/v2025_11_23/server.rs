use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub webroot: Option<PathBuf>,
    pub jwt_secret_file: PathBuf,
    pub data_dir: PathBuf,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            data_dir: PathBuf::from("/data/peer_practice"),
            port: 3000,
            webroot: Some(PathBuf::from("web-leptos/dist")),
            jwt_secret_file: PathBuf::from("/super/secret/jwt_secret_file.txt"),
            cors_allowed_origins: vec![
                "http://localhost".to_string(),
                "https://localhost".to_string(),
            ],
        }
    }
}

impl TryFrom<ServerConfig> for crate::input::config::current::server::ServerConfig {
    type Error = eyre::Error;
    fn try_from(value: ServerConfig) -> Result<Self, Self::Error> {
        let jwt_secret = std::fs::read_to_string(&value.jwt_secret_file)?;
        Ok(Self {
            jwt_secret,
            data_dir: value.data_dir,
            port: value.port,
            webroot: value.webroot,
            cors_allowed_origins: value.cors_allowed_origins,
        })
    }
}

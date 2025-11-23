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

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            data_dir: PathBuf::from("/data/peer_practice"),
            port: 3000,
            webroot: Some(PathBuf::from("web-leptos/dist")),
            jwt_secret: "change-me-jwt-secret".to_string(),
            cors_allowed_origins: vec![
                "http://localhost".to_string(),
                "https://localhost".to_string(),
            ],
        }
    }
}

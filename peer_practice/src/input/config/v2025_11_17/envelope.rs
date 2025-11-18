use crate::input::config::ConfigVersion;
use super::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct V2025_11_17Config {
    pub version: ConfigVersion,
    #[serde(flatten)]
    pub config: Config,
}

impl Default for V2025_11_17Config {
    fn default() -> Self {
        Self {
            version: ConfigVersion::V2025_11_17,
            config: Config::default(),
        }
    }
}
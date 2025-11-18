use crate::input::config::current::Config;
use crate::input::config::v2025_11_17::envelope::V2025_11_17Config;

pub mod v2025_11_17;

pub mod current {
    pub use super::v2025_11_17::server;
    pub use super::v2025_11_17::envelope;
    pub use super::v2025_11_17::Config;
    pub use super::v2025_11_17::email;
    pub type Envelope = envelope::V2025_11_17Config;

}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConfigEnvelop {
    version: ConfigVersion,
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConfigVersion {
    V2025_11_17,
}

impl ConfigEnvelop {
    pub(crate) fn config(file_content: &str) -> Result<Config, eyre::Error> {
        let config_envelop: ConfigEnvelop = toml::from_str(file_content)?;
        match config_envelop.version {
            ConfigVersion::V2025_11_17 => {
                let v2025_11_17_config: V2025_11_17Config = toml::from_str(file_content)?;
                Ok(v2025_11_17_config.config)
            }
        }
    }
}
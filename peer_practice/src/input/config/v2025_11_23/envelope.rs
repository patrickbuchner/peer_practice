use super::Config;
use crate::input::config::{ConfigEnvelopeExt, ConfigVersion};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct V2025_11_23Config {
    pub version: ConfigVersion,
    #[serde(flatten)]
    pub config: Config,
}
impl ConfigEnvelopeExt for V2025_11_23Config {
    fn config(&self) -> Result<crate::input::config::current::Config, eyre::Error> {
        self.config.clone().try_into()
    }
}

impl Default for V2025_11_23Config {
    fn default() -> Self {
        Self {
            version: ConfigVersion::V2025_11_23,
            config: Config::default(),
        }
    }
}

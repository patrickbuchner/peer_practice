use crate::input::config::current::Config;
use crate::input::config::v2025_11_17::envelope::V2025_11_17Config;
use crate::input::config::v2025_11_23::envelope::V2025_11_23Config;
use eyre::WrapErr;

pub mod current;
pub mod v2025_11_17;
pub mod v2025_11_23;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConfigEnvelope {
    version: ConfigVersion,
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConfigVersion {
    V2025_11_17,
    V2025_11_23,
}

trait ConfigEnvelopeExt {
    fn config(&self) -> Result<Config, eyre::Error>;
}

impl ConfigEnvelope {
    pub(crate) fn config(file_content: &str) -> Result<Config, eyre::Error> {
        let config_envelop: ConfigEnvelope =
            toml::from_str(file_content).wrap_err("Unknown config version.")?;
        let config_envelope: &dyn ConfigEnvelopeExt = match config_envelop.version {
            ConfigVersion::V2025_11_17 => &toml::from_str::<V2025_11_17Config>(file_content)?,
            ConfigVersion::V2025_11_23 => &toml::from_str::<V2025_11_23Config>(file_content)?,
        };

        config_envelope.config()
    }
}

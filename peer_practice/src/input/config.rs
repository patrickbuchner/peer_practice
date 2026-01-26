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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::config::v2025_11_17::envelope::V2025_11_17Config;
    use crate::input::config::v2025_11_17::{
        Config as Config17, email::EmailConfig as Email17, server::ServerConfig as Server17,
    };
    use crate::input::config::v2025_11_23::envelope::V2025_11_23Config;
    use crate::input::config::v2025_11_23::{
        Config as Config23, email::EmailConfig as Email23, server::ServerConfig as Server23,
    };
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_file(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("peer_practice_{name}_{}", Uuid::new_v4()))
    }

    #[test]
    fn parses_v2025_11_17_config() {
        let envelope = V2025_11_17Config {
            version: ConfigVersion::V2025_11_17,
            config: Config17 {
                email: Email17 {
                    from: "from@example.com".to_string(),
                    reply_to: "reply@example.com".to_string(),
                    tls_relay: "smtp.example.com:587".to_string(),
                    credential_email_account: "user@example.com".to_string(),
                    password: "secret".to_string(),
                },
                server: Server17 {
                    webroot: Some(PathBuf::from("webroot")),
                    jwt_secret: "jwt-secret".to_string(),
                    data_dir: PathBuf::from("/tmp/peer-practice"),
                    port: 1234,
                    cors_allowed_origins: vec!["http://localhost".to_string()],
                },
            },
        };

        let toml = toml::to_string(&envelope).expect("serialize");
        let config = ConfigEnvelope::config(&toml).expect("parse");
        assert_eq!(config.server.jwt_secret, "jwt-secret");
        assert_eq!(config.server.port, 1234);
        assert_eq!(config.email.credential_email_account, "user@example.com");
    }

    #[test]
    fn parses_v2025_11_23_config_with_file_secrets() {
        let jwt_file = temp_file("jwt");
        let pass_file = temp_file("email_pass");
        fs::write(&jwt_file, "jwt-from-file").expect("write jwt");
        fs::write(&pass_file, "pass-from-file").expect("write pass");

        let envelope = V2025_11_23Config {
            version: ConfigVersion::V2025_11_23,
            config: Config23 {
                email: Email23 {
                    from: "from@example.com".to_string(),
                    reply_to: "reply@example.com".to_string(),
                    tls_relay: "smtp.example.com:587".to_string(),
                    credential_email_account: "user@example.com".to_string(),
                    password_file: pass_file.clone(),
                },
                server: Server23 {
                    webroot: Some(PathBuf::from("webroot")),
                    jwt_secret_file: jwt_file.clone(),
                    data_dir: PathBuf::from("/tmp/peer-practice"),
                    port: 4321,
                    cors_allowed_origins: vec!["http://localhost".to_string()],
                },
            },
        };

        let toml = toml::to_string(&envelope).expect("serialize");
        let config = ConfigEnvelope::config(&toml).expect("parse");
        assert_eq!(config.server.jwt_secret, "jwt-from-file");
        assert_eq!(config.email.password, "pass-from-file");

        let _ = fs::remove_file(jwt_file);
        let _ = fs::remove_file(pass_file);
    }

    #[test]
    fn unknown_version_is_error() {
        let toml = r#"
version = "V2099_01_01"
"#;
        let err = ConfigEnvelope::config(toml);
        assert!(err.is_err(), "expected unknown version error");
    }
}

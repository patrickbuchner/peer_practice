use eyre::WrapErr;
use peer_practice_server_services::email::EmailConfiguration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub from: String,
    pub reply_to: String,
    pub tls_relay: String,
    pub credential_email_account: String,
    pub password_file: PathBuf,
}
impl TryFrom<EmailConfig> for EmailConfiguration {
    type Error = eyre::Error;
    fn try_from(value: EmailConfig) -> Result<Self, Self::Error> {
        let password = std::fs::read_to_string(&value.password_file)?;
        EmailConfiguration::new(
            value.tls_relay,
            value.credential_email_account,
            password,
            &value.from,
            &value.reply_to,
        )
        .with_context(|| "Could not create email configuration.")
    }
}

impl Default for EmailConfig {
    fn default() -> Self {
        EmailConfig {
            tls_relay: "smtp.gmail.com:587".to_string(),
            password_file: PathBuf::from("/very/good/protected/email_password.txt"),
            credential_email_account: "for@example.com".to_string(),
            reply_to: "replay.to@example.com".to_string(),
            from: "from@example.com".to_string(),
        }
    }
}

impl TryFrom<EmailConfig> for crate::input::config::current::email::EmailConfig {
    type Error = eyre::Error;
    fn try_from(value: EmailConfig) -> Result<Self, Self::Error> {
        let password =
            std::fs::read_to_string(&value.password_file)?;
        Ok(Self {
            from: value.from,
            reply_to: value.reply_to,
            password,
            tls_relay: value.tls_relay,
            credential_email_account: value.credential_email_account,
        })
    }
}

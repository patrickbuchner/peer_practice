use eyre::WrapErr;
use peer_practice_server_services::email::EmailConfiguration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub from: String,
    pub reply_to: String,
    pub tls_relay: String,
    pub credential_email_account: String,
    pub password: String,
}
impl TryFrom<EmailConfig> for EmailConfiguration {
    type Error = eyre::Error;
    fn try_from(value: EmailConfig) -> Result<Self, Self::Error> {
        EmailConfiguration::new(
            value.tls_relay,
            value.credential_email_account,
            value.password,
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
            password: "change-me-email-password".to_string(),
            credential_email_account: "for@example.com".to_string(),
            reply_to: "replay.to@example.com".to_string(),
            from: "from@example.com".to_string(),
        }
    }
}

use eyre::WrapErr;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::str::FromStr;
use tokio::sync::{mpsc, oneshot};

pub enum EmailMsg {
    SendLoginMail {
        target: Mailbox,
        validation_code: u32,
        respond_to: oneshot::Sender<Result<Response, eyre::Error>>,
    },
}

pub struct EmailConfiguration {
    pub credentials: Credentials,
    pub from: Mailbox,
    pub reply_to: Mailbox,
    pub tls_relay: String
}

impl EmailConfiguration {
    pub fn new(
        tls_relay: String,
        credentials: String,
        password: String,
        from: &str,
        reply_to: &str,
    ) -> Result<Self, eyre::Error> {
        Ok(Self {
            reply_to: Mailbox::from_str(reply_to)
                .wrap_err("Could not parse \"reply_to\" email address.")?,
            from: Mailbox::from_str(from).wrap_err("Could not parse \"from\" email address.")?,
            credentials: Credentials::new(credentials, password),
            tls_relay
        })
    }
}

pub fn spawn_email_actor(config: EmailConfiguration) -> mpsc::Sender<EmailMsg> {
    let (tx, mut rx) = mpsc::channel::<EmailMsg>(64);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                EmailMsg::SendLoginMail {
                    target,
                    validation_code,
                    respond_to,
                } => {
                    let res = send_login_mail(&config, target, validation_code).await;
                    let _ = respond_to.send(res);
                }
            }
        }
    });

    tx
}

async fn send_login_mail(
    config: &EmailConfiguration,
    target: impl Into<Mailbox>,
    validation_code: u32,
) -> Result<Response, eyre::Error> {
    let email = Message::builder()
        .from(config.from.clone())
        .reply_to(config.reply_to.clone())
        .to(target.into())
        .subject(format!("Login Code {validation_code}"))
        .header(ContentType::TEXT_PLAIN)
        .body(format!("{validation_code}"))
        .with_context(|| "Could not create email.")?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.tls_relay)?
        .credentials(config.credentials.clone())
        .build();

    // Send the email
    mailer.send(email).await.wrap_err("Failed to send email.")
}

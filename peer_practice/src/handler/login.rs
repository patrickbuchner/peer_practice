use crate::app_state::AppState;
use crate::handler::claims::Claims;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use peer_practice_messages::current::authentication::login_data::{LoginData, PinLogin};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::email::EmailMsg;
use peer_practice_server_services::pending_logins::PendingLoginsMsg;
use peer_practice_server_services::users::UsersMsg;
use rand::prelude::*;
use tokio::sync::oneshot;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::OffsetDateTime;
use peer_practice_messages::current::sessions::SessionId;

#[axum::debug_handler]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(login_data): Json<LoginData>,
) -> Result<Json<Option<UserId>>, StatusCode> {
    // Request user by email
    let (tx_user, rx_user) = oneshot::channel();
    let _ = state
        .users
        .send(UsersMsg::GetByEmail {
            email: login_data.email.clone(),
            respond_to: tx_user,
        })
        .await;

    // Generate 6-digit PIN
    let pin: u32 = {
        let mut rng = rand::rng();
        rng.random_range(100_000..=999_999)
    };

    // Store or update pending login
    let _ = state
        .pending_logins
        .send(PendingLoginsMsg::Upsert {
            address: login_data.email.clone(),
            code: pin,
        })
        .await;

    // Send login email (ignore result, but keep TODO note)
    let (tx_mail, _rx_mail) = oneshot::channel();
    let _ = state
        .email
        .send(EmailMsg::SendLoginMail {
            respond_to: tx_mail,
            target: login_data.email.clone().into(),
            validation_code: pin,
        })
        .await;
    // TODO: consider logging the email send result from _rx_mail

    // Return existing user id (or None) if lookup succeeded
    rx_user
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn pin_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(pin_login): Json<PinLogin>,
) -> Result<CookieJar, StatusCode> {
    let (tx_user, rx_user) = oneshot::channel();
    let _ = state
        .users
        .send(UsersMsg::GetById {
            id: pin_login.id,
            respond_to: tx_user,
        })
        .await;

    let user = rx_user
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let (tx_pin, rx_pin) = oneshot::channel();
    let _ = state
        .pending_logins
        .send(PendingLoginsMsg::GetByAddress {
            address: user.email,
            respond_to: tx_pin,
        })
        .await;

    let stored_pin = rx_pin
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let provided_pin: u32 = pin_login
        .pin
        .parse()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if provided_pin != stored_pin {
        return Err(StatusCode::UNAUTHORIZED);
    }

    create_access_cookie(&state, jar, pin_login.id, None)
}

pub fn create_access_cookie(
    state: &AppState,
    jar: CookieJar,
    user_id: UserId,
    id: Option<SessionId>,
) -> Result<CookieJar, StatusCode> {
    let access_claims = Claims {
        user_id,
        exp: (Utc::now() + state.jwt_expiry_duration).timestamp() as usize,
        client_id: id,
    };
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = jar.add(
        Cookie::build(("access_token", access_token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(
                OffsetDateTime::now_utc()
                    + tower_sessions::cookie::time::Duration::days(
                        state.jwt_expiry_duration.num_days(),
                    ),
            ),
    );
    Ok(jar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::test_state;
    use peer_practice_messages::current::authentication::method::AuthenticationMethod;
    use peer_practice_messages::current::email::Email;
    use peer_practice_messages::current::user::User;
    use peer_practice_server_services::email::EmailMsg;
    use peer_practice_server_services::pending_logins::PendingLoginsMsg;
    use peer_practice_server_services::users::UsersMsg;

    async fn recv_msg<T>(rx: &mut tokio::sync::mpsc::Receiver<T>) -> T {
        match rx.recv().await {
            Some(msg) => msg,
            None => panic!("channel closed"),
        }
    }

    fn sample_user(id: UserId, email: Email) -> User {
        User {
            id,
            email,
            display_name: Some("Tester".to_string()),
        }
    }

    #[tokio::test]
    async fn login_handler_sends_pin_and_returns_user_id() {
        let (state, mut rx) = test_state();
        let email = Email::new("user@example.com").unwrap();
        let login = LoginData {
            email: email.clone(),
            auth: AuthenticationMethod::EmailOTP,
        };
        let user_id = UserId::new();

        let handler = tokio::spawn(login_handler(State(state), Json(login)));

        match recv_msg(&mut rx.users).await {
            UsersMsg::GetByEmail {
                email: got_email,
                respond_to,
            } => {
                assert_eq!(email.value(), got_email.value());
                let _ = respond_to.send(Some(user_id));
            }
            _ => panic!("expected UsersMsg::GetByEmail"),
        }

        let pin = match recv_msg(&mut rx.pending_logins).await {
            PendingLoginsMsg::Upsert { address, code } => {
                assert_eq!(email.value(), address.value());
                assert!((100_000..=999_999).contains(&code));
                code
            }
            _ => panic!("expected PendingLoginsMsg::Upsert"),
        };

        match recv_msg(&mut rx.email).await {
            EmailMsg::SendLoginMail {
                target,
                validation_code,
                respond_to: _,
            } => {
                assert_eq!(email.value(), target.to_string());
                assert_eq!(pin, validation_code);
            }
        }

        let result = handler.await.expect("handler task ok");
        let Json(got) = result.expect("handler ok");
        assert_eq!(Some(user_id), got);
    }

    #[tokio::test]
    async fn pin_handler_sets_access_cookie_on_success() {
        let (state, mut rx) = test_state();
        let email = Email::new("user@example.com").unwrap();
        let user_id = UserId::new();
        let pin = 123_456u32;

        let handler = tokio::spawn(pin_handler(
            State(state),
            CookieJar::new(),
            Json(PinLogin {
                pin: pin.to_string(),
                id: user_id,
            }),
        ));

        match recv_msg(&mut rx.users).await {
            UsersMsg::GetById { id, respond_to } => {
                assert_eq!(user_id, id);
                let _ = respond_to.send(Some(sample_user(user_id, email.clone())));
            }
            _ => panic!("expected UsersMsg::GetById"),
        }

        match recv_msg(&mut rx.pending_logins).await {
            PendingLoginsMsg::GetByAddress {
                address,
                respond_to,
            } => {
                assert_eq!(email.value(), address.value());
                let _ = respond_to.send(Some(pin));
            }
            _ => panic!("expected PendingLoginsMsg::GetByAddress"),
        }

        let jar = handler.await.expect("handler task ok").expect("handler ok");
        let cookie = jar.get("access_token").expect("missing access_token");
        assert!(!cookie.value().is_empty());
    }
}

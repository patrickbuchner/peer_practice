use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use tokio::sync::oneshot;

use crate::app_state::AppState;
use crate::handler::claims::Claims;
use peer_practice_server_services::email::EmailMsg;
use peer_practice_server_services::pending_logins::PendingLoginsMsg;
use peer_practice_server_services::users::UsersMsg;
use peer_practice_shared::authentication::login_data::{LoginData, PinLogin};
use peer_practice_shared::user::UserId;
use rand::prelude::*;
use tower_sessions::cookie::time::OffsetDateTime;

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
    
    let offset = 15;
    let access_claims = Claims {
        user_id: pin_login.id,
        exp: (Utc::now() + Duration::days(offset)).timestamp() as usize,
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
            .expires(
                OffsetDateTime::now_utc() + tower_sessions::cookie::time::Duration::days(offset),
            ),
    );

    Ok(jar)
}

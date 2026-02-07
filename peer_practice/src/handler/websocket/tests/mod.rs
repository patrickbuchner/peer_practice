use super::*;
use crate::handler::test_utils::test_state;
use chrono::TimeZone;
use peer_practice_messages::current::level::Level;
use peer_practice_messages::current::post::{Post, Topics};
use peer_practice_messages::current::user::display_user::UserDisplay;
use peer_practice_messages::current::chat::{ChatId, ChatMessage};
use std::collections::HashSet;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot;
use uuid::Uuid;

enum ExpectedResult {
    Ok(Version, ClientToServer),
    Err,
}

macro_rules! parse_case {
    ($name:ident, $input:expr, $expected_result:expr) => {
        #[test]
        fn $name() {
            let result = parse_received_message(&Utf8Bytes::from($input));
            match (result, $expected_result) {
                (Ok(actual), ExpectedResult::Ok(expected_version, expected_data)) => {
                    assert_eq!(actual.0, expected_version);
                    assert_eq!(actual.1, expected_data);
                }
                (Err(_), ExpectedResult::Err) => {}
                _ => panic!("Unexpected result"),
            }
        }
    };
}

mod proptest;
mod v2025_10_14;
mod v2026_01_11;
mod v2026_02_07;

fn sample_user_display(user_id: UserId) -> UserDisplay {
    UserDisplay {
        display_name: Some("Alice".into()),
        id: user_id,
    }
}

fn sample_post(user_id: UserId) -> Post {
    let mut partaking_users = HashSet::new();
    partaking_users.insert(user_id);

    Post {
        title: Topics::Basics,
        content: "Looking for practice partners.".into(),
        level: Level::Beginner1,
        owner: user_id,
        date: chrono::Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap(),
        partaking_users,
    }
}

fn sample_chat_id() -> ChatId {
    ChatId::from_id(Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap())
}

fn sample_chat_message(user_id: UserId, chat_id: ChatId) -> ChatMessage {
    ChatMessage {
        sender: user_id,
        kind: peer_practice_messages::current::chat::ChatMessageKind::Text(
            "Hello from chat.".into(),
        ),
        chat_id,
    }
}

async fn sync_ws_hub(state: &AppState, rx: &mut tokio::sync::mpsc::Receiver<WsHubMsg>) {
    let (respond_to, recv) = oneshot::channel();
    state
        .ws_hub
        .send(WsHubMsg::Ping { respond_to })
        .await
        .expect("send ping");

    let msg = rx.recv().await.expect("channel closed");
    match msg {
        WsHubMsg::Ping { respond_to } => {
            let _ = respond_to.send(());
        }
        _ => panic!("expected WsHubMsg::Ping"),
    }

    recv.await.expect("ping ack");
}

fn assert_empty<T>(rx: &mut tokio::sync::mpsc::Receiver<T>) {
    match rx.try_recv() {
        Ok(_) => panic!("expected no message"),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => panic!("channel closed"),
    }
}

#[test]
fn serialize_server_message_sets_expected_version() {
    let msg = ServerToClient::MessageNotYetKnown;

    let current = serialize_server_message(&msg, Version::V2026_02_07).expect("serialize current");
    let header: EnvelopeHeader = serde_json::from_str(&current).expect("parse header");
    assert_eq!(Version::V2026_02_07, header.version);

    let prev = serialize_server_message(&msg, Version::V2026_01_11).expect("serialize previous");
    let header: EnvelopeHeader = serde_json::from_str(&prev).expect("parse header");
    assert_eq!(Version::V2026_01_11, header.version);

    let legacy = serialize_server_message(&msg, Version::V2025_10_14).expect("serialize legacy");
    let header: EnvelopeHeader = serde_json::from_str(&legacy).expect("parse header");
    assert_eq!(Version::V2025_10_14, header.version);
}

#[tokio::test]
async fn consume_telegram_rejects_version_mismatch() {
    let (state, mut rx) = test_state();
    let user_id = UserId::new();
    let con_id = ConnectionId::new();

    let text = serde_json::to_string(&Envelope {
        version: Version::V2025_10_14,
        data: peer_practice_messages::v2025_10_14::messages::ClientToServer::Hello,
    })
    .expect("serialize envelope");

    let res = consume_telegram(
        &Some(Version::V2026_02_07),
        &Utf8Bytes::from(text),
        con_id,
        user_id,
        &state,
    )
    .await;
    assert!(res.is_err(), "expected version mismatch error");

    sync_ws_hub(&state, &mut rx.ws_hub).await;
    assert_empty(&mut rx.ws_hub);
}

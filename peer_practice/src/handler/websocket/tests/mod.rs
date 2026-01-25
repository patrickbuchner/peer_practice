use super::*;
use chrono::TimeZone;
use peer_practice_messages::current::level::Level;
use peer_practice_messages::current::post::{Post, Topics};
use peer_practice_messages::current::user::display_user::UserDisplay;
use peer_practice_messages::v2026_01_11::chat::{ChatId, ChatMessage};
use std::collections::HashSet;
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

mod v2025_10_14;
mod v2026_01_11;
mod proptest;

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
        message: "Hello from chat.".into(),
        chat_id,
    }
}

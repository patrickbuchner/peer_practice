use super::super::*;
use axum::extract::ws::Utf8Bytes;
use chrono::TimeZone;
use peer_practice_messages::v2025_10_14::level::Level;
use peer_practice_messages::v2025_10_14::messages::ClientToServer as LegacyClientToServer;
use peer_practice_messages::v2025_10_14::post::{Post, PostId, Topics};
use peer_practice_messages::v2025_10_14::user::UserId;
use peer_practice_messages::v2025_10_14::user::display_user::UserDisplay;
use peer_practice_messages::v2026_01_11::chat::{ChatId, ChatMessage};
use peer_practice_messages::v2026_01_11::messages::ClientToServer as CurrentClientToServer;
use peer_practice_messages::v2026_01_11::messages::client_to_server::{
    ChatAction, PostAction, UserAction,
};
use proptest::collection::hash_set;
use proptest::prelude::*;
use uuid::Uuid;

fn ascii_string(max_len: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(proptest::char::range(' ', '~'), 0..=max_len)
        .prop_map(|chars| chars.into_iter().collect())
}

fn user_id_from_uuid(uuid: Uuid) -> UserId {
    serde_json::from_str(&format!(r#"{{"id":"{}"}}"#, uuid)).expect("valid user id")
}

fn post_id_from_uuid(uuid: Uuid) -> PostId {
    serde_json::from_str(&format!(r#"{{"id":"{}"}}"#, uuid)).expect("valid post id")
}

prop_compose! {
    fn user_id_strategy()(bytes in any::<[u8; 16]>()) -> UserId {
        user_id_from_uuid(Uuid::from_bytes(bytes))
    }
}

prop_compose! {
    fn post_id_strategy()(bytes in any::<[u8; 16]>()) -> PostId {
        post_id_from_uuid(Uuid::from_bytes(bytes))
    }
}

prop_compose! {
    fn chat_id_strategy()(bytes in any::<[u8; 16]>()) -> ChatId {
        ChatId::from_id(Uuid::from_bytes(bytes))
    }
}

prop_compose! {
    fn user_display_strategy()
        (id in user_id_strategy(), display_name in prop::option::of(ascii_string(32))) -> UserDisplay {
        UserDisplay { display_name, id }
    }
}

prop_compose! {
    fn date_time_strategy()(secs in 0i64..=4_102_444_800i64) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.timestamp_opt(secs, 0).single().expect("valid timestamp")
    }
}

prop_compose! {
    fn post_strategy()
        (
            title in prop::sample::select(Topics::ALL),
            content in ascii_string(200),
            level in prop::sample::select(Level::ALL),
            owner in user_id_strategy(),
            date in date_time_strategy(),
            partaking_users in hash_set(user_id_strategy(), 0..5),
        ) -> Post {
        Post {
            title,
            content,
            level,
            owner,
            date,
            partaking_users,
        }
    }
}

prop_compose! {
    fn chat_message_strategy()
        (sender in user_id_strategy(), message in ascii_string(200), chat_id in chat_id_strategy())
        -> ChatMessage {
        ChatMessage {
            sender,
            message,
            chat_id,
        }
    }
}

fn legacy_client_to_server_strategy() -> impl Strategy<Value = LegacyClientToServer> {
    prop_oneof![
        Just(LegacyClientToServer::MessageNotYetKnown),
        Just(LegacyClientToServer::Hello),
        user_id_strategy().prop_map(LegacyClientToServer::GetUser),
        user_display_strategy().prop_map(LegacyClientToServer::UpdateUser),
        Just(LegacyClientToServer::GetPosts),
        post_id_strategy().prop_map(LegacyClientToServer::Join),
        post_id_strategy().prop_map(LegacyClientToServer::Leave),
        (post_id_strategy(), post_strategy())
            .prop_map(|(id, post)| LegacyClientToServer::UpdatePost(id, post)),
        post_strategy().prop_map(LegacyClientToServer::NewPost),
        post_id_strategy().prop_map(LegacyClientToServer::DeletePost),
    ]
}

fn current_client_to_server_strategy() -> impl Strategy<Value = CurrentClientToServer> {
    let user_action = prop_oneof![
        user_id_strategy().prop_map(UserAction::Get),
        user_display_strategy().prop_map(UserAction::Update),
    ];

    let post_action = prop_oneof![
        Just(PostAction::GetPosts),
        post_id_strategy().prop_map(PostAction::Join),
        post_id_strategy().prop_map(PostAction::Leave),
        (post_id_strategy(), post_strategy())
            .prop_map(|(id, post)| PostAction::UpdatePost(id, post)),
        post_strategy().prop_map(PostAction::NewPost),
        post_id_strategy().prop_map(PostAction::DeletePost),
        post_id_strategy().prop_map(PostAction::GetPostMessages),
    ];

    let chat_action = prop_oneof![
        post_id_strategy().prop_map(ChatAction::GetChatFor),
        chat_id_strategy().prop_map(ChatAction::GetChat),
        chat_message_strategy().prop_map(ChatAction::SendMessage),
    ];

    prop_oneof![
        Just(CurrentClientToServer::Hello),
        Just(CurrentClientToServer::MessageNotYetKnown),
        user_action.prop_map(CurrentClientToServer::User),
        post_action.prop_map(CurrentClientToServer::Post),
        chat_action.prop_map(CurrentClientToServer::Chat),
    ]
}

proptest! {
    #[test]
    fn parse_current_client_messages_roundtrip(message in current_client_to_server_strategy()) {
        let text = serde_json::to_string(&Envelope {
            version: Version::V2026_01_11,
            data: &message,
        })
        .expect("serialize envelope");
        let envelope = Envelope {
            version: Version::V2026_01_11,
            data: message,
        };
        let parsed = parse_received_message(&Utf8Bytes::from(text)).expect("parse message");

        prop_assert_eq!(parsed.0, Version::V2026_01_11);
        prop_assert_eq!(parsed.1, envelope.data);
    }

    #[test]
    fn parse_legacy_client_messages_roundtrip(message in legacy_client_to_server_strategy()) {
        let text = serde_json::to_string(&Envelope {
            version: Version::V2025_10_14,
            data: &message,
        })
        .expect("serialize envelope");
        let expected: CurrentClientToServer = message.into();
        let parsed = parse_received_message(&Utf8Bytes::from(text)).expect("parse message");

        prop_assert_eq!(parsed.0, Version::V2025_10_14);
        prop_assert_eq!(parsed.1, expected);
    }
}

use super::super::*;
use axum::extract::ws::Utf8Bytes;
use chrono::TimeZone;
use peer_practice_messages::current::chat::{ChatId, ChatMessage};
use peer_practice_messages::current::messages::ClientToServer as CurrentClientToServer;
use peer_practice_messages::current::messages::client_to_server::{
    ChatAction, PostAction, SessionAction, UserAction,
};
use peer_practice_messages::current::sessions::{SessionId, SessionInformation};
use peer_practice_messages::v2025_10_14::level::Level;
use peer_practice_messages::v2025_10_14::messages::ClientToServer as LegacyClientToServer;
use peer_practice_messages::v2025_10_14::post::{Post, PostId, Topics};
use peer_practice_messages::v2025_10_14::user::UserId;
use peer_practice_messages::v2025_10_14::user::display_user::UserDisplay;
use proptest::collection::hash_set;
use proptest::prelude::*;
use uuid::Uuid;

fn ascii_string(max_len: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(proptest::char::range(' ', '~'), 0..=max_len)
        .prop_map(|chars| chars.into_iter().collect())
}

fn current_user_id_from_uuid(uuid: Uuid) -> UserId {
    serde_json::from_str(&format!(r#"{{"id":"{}"}}"#, uuid)).expect("valid current user id")
}

fn current_post_id_from_uuid(uuid: Uuid) -> PostId {
    serde_json::from_str(&format!(r#"{{"id":"{}"}}"#, uuid)).expect("valid current post id")
}

prop_compose! {
    fn current_user_id_strategy()(bytes in any::<[u8; 16]>()) -> UserId {
        current_user_id_from_uuid(Uuid::from_bytes(bytes))
    }
}

prop_compose! {
    fn current_post_id_strategy()(bytes in any::<[u8; 16]>()) -> PostId {
        current_post_id_from_uuid(Uuid::from_bytes(bytes))
    }
}

fn current_server_to_client_strategy() -> impl Strategy<Value = ServerToClient> {
    use peer_practice_messages::current::messages::server_to_client as stc;

    prop_oneof![
        Just(ServerToClient::MessageNotYetKnown),
        current_user_id_strategy()
            .prop_map(|id| { ServerToClient::User(stc::UserAction::YouAre(id)) }),
        current_post_id_strategy().prop_map(|post_id| {
            ServerToClient::Chat(stc::ChatAction::ChatDoesNotExistForPost(post_id))
        }),
    ]
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
            kind: peer_practice_messages::current::chat::ChatMessageKind::Text(message),
            chat_id,
        }
    }
}

fn session_id_from_uuid(uuid: Uuid) -> SessionId {
    serde_json::from_str(&format!(r#"{{"id":"{}"}}"#, uuid)).expect("valid session id")
}

prop_compose! {
    fn session_id_strategy()(bytes in any::<[u8; 16]>()) -> SessionId {
        session_id_from_uuid(Uuid::from_bytes(bytes))
    }
}

prop_compose! {
    fn session_information_strategy()
        (session_id in session_id_strategy(), description in ascii_string(64))
        -> SessionInformation {
        SessionInformation { session_id, description }
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
    ];

    let chat_action = prop_oneof![
        post_id_strategy().prop_map(ChatAction::GetChatFor),
        chat_id_strategy().prop_map(ChatAction::GetChat),
        chat_message_strategy().prop_map(ChatAction::SendMessage),
    ];

    let session_action = prop_oneof![
        Just(SessionAction::GetSessions),
        Just(SessionAction::GetThisSession),
        session_information_strategy().prop_map(SessionAction::UpdateSession),
        session_id_strategy().prop_map(SessionAction::LogOutSession),
        Just(SessionAction::LogOutAllSessions),
    ];

    prop_oneof![
        Just(CurrentClientToServer::Hello),
        Just(CurrentClientToServer::MessageNotYetKnown),
        user_action.prop_map(CurrentClientToServer::User),
        post_action.prop_map(CurrentClientToServer::Post),
        chat_action.prop_map(CurrentClientToServer::Chat),
        session_action.prop_map(CurrentClientToServer::Session),
    ]
}

fn assert_serde_json_value_eq<T: serde::Serialize, U: serde::Serialize>(left: &T, right: &U) {
    let left = serde_json::to_value(left).expect("to_value(left)");
    let right = serde_json::to_value(right).expect("to_value(right)");
    assert_eq!(left, right);
}

proptest! {
    #[test]
    fn parse_current_client_messages_roundtrip(message in current_client_to_server_strategy()) {
        let text = serde_json::to_string(&Envelope {
            version: Version::V2026_02_07,
            data: &message,
        })
        .expect("serialize envelope");
        let envelope = Envelope {
            version: Version::V2026_02_07,
            data: message,
        };
        let parsed = parse_received_message(&Utf8Bytes::from(text)).expect("parse message");

        prop_assert_eq!(parsed.0, Version::V2026_02_07);
        prop_assert_eq!(parsed.1, envelope.data);
    }

    #[test]
    fn parse_v2026_01_11_client_messages_upgrade_to_current(
        message in current_client_to_server_strategy()
    ) {
        let v01: peer_practice_messages::v2026_01_11::messages::ClientToServer =
            message.clone().into();

        let expected_current: CurrentClientToServer = v01.clone().into();

        let text = serde_json::to_string(&Envelope {
            version: Version::V2026_01_11,
            data: &v01,
        })
        .expect("serialize envelope");

        let parsed = parse_received_message(&Utf8Bytes::from(text)).expect("parse message");

        prop_assert_eq!(parsed.0, Version::V2026_01_11);
        prop_assert_eq!(parsed.1, expected_current);
    }

    #[test]
    fn parse_legacy_client_messages_roundtrip(message in legacy_client_to_server_strategy()) {
        let text = serde_json::to_string(&Envelope {
            version: Version::V2025_10_14,
            data: &message,
        })
        .expect("serialize envelope");
        let expected: peer_practice_messages::v2026_01_11::messages::ClientToServer = message.into();
        let expected: CurrentClientToServer = expected.into();

        let parsed = parse_received_message(&Utf8Bytes::from(text)).expect("parse message");

        prop_assert_eq!(parsed.0, Version::V2025_10_14);
        prop_assert_eq!(parsed.1, expected);
    }

    #[test]
    fn serialize_current_server_messages_roundtrip_current(
        message in current_server_to_client_strategy()
    ) {
        let text = serialize_server_message(&message, Version::V2026_02_07).expect("serialize");
        let env: Envelope<ServerToClient> =
            serde_json::from_str(&text).expect("parse envelope");

        prop_assert_eq!(env.version, Version::V2026_02_07);

        assert_serde_json_value_eq(&env.data, &message);
    }

    #[test]
    fn serialize_current_server_messages_downconvert_to_v2026_01_11(
        message in current_server_to_client_strategy()
    ) {
        let expected_current_after_roundtrip: ServerToClient = {
            let v01: peer_practice_messages::v2026_01_11::messages::ServerToClient = message.clone().into();
            v01.into()
        };

        let text = serialize_server_message(&message, Version::V2026_01_11).expect("serialize");
        let env: Envelope<peer_practice_messages::v2026_01_11::messages::ServerToClient> =
            serde_json::from_str(&text).expect("parse envelope");

        prop_assert_eq!(env.version, Version::V2026_01_11);

        let got_current: ServerToClient = env.data.into();
        assert_serde_json_value_eq(&got_current, &expected_current_after_roundtrip);
    }

    #[test]
    fn serialize_current_server_messages_downconvert_to_v2025_10_14(
        message in current_server_to_client_strategy()
    ) {
        let expected_current_after_roundtrip: ServerToClient = {
            let v01: peer_practice_messages::v2026_01_11::messages::ServerToClient = message.clone().into();
            let v10: peer_practice_messages::v2025_10_14::messages::ServerToClient = v01.into();
            let v01_back: peer_practice_messages::v2026_01_11::messages::ServerToClient = v10.into();
            v01_back.into()
        };

        let text = serialize_server_message(&message, Version::V2025_10_14).expect("serialize");
        let env: Envelope<peer_practice_messages::v2025_10_14::messages::ServerToClient> =
            serde_json::from_str(&text).expect("parse envelope");

        prop_assert_eq!(env.version, Version::V2025_10_14);

        let v01: peer_practice_messages::v2026_01_11::messages::ServerToClient = env.data.into();
        let got_current: ServerToClient = v01.into();

        assert_serde_json_value_eq(&got_current, &expected_current_after_roundtrip);
    }
}

use super::*;
mod valid_parsing {
    use super::*;
    use peer_practice_messages::current::messages::client_to_server::{
        ChatAction, PostAction, UserAction,
    };
    use peer_practice_messages::current::post::PostId;

    parse_case!(
        hello,
        r#"{
      "version": "V2026_02_07",
      "data": "Hello"
    }"#,
        ExpectedResult::Ok(Version::V2026_02_07, ClientToServer::Hello)
    );

    parse_case!(
        message_not_yet_known,
        r#"{
      "version": "V2026_02_07",
      "data": "MessageNotYetKnown"
    }"#,
        ExpectedResult::Ok(Version::V2026_02_07, ClientToServer::MessageNotYetKnown)
    );

    parse_case!(
        user_get,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "User": {
          "Get": {
            "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::User(UserAction::Get(UserId::test()))
        )
    );

    parse_case!(
        user_update,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "User": {
          "Update": {
            "display_name": "Alice",
            "id": {
              "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
            }
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::User(UserAction::Update(sample_user_display(UserId::test())))
        )
    );

    parse_case!(
        post_get_posts,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": "GetPosts"
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::GetPosts)
        )
    );

    parse_case!(
        post_join,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": {
          "Join": {
            "id": "00000000-0000-0000-0000-000000000000"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::Join(PostId::NULL))
        )
    );

    parse_case!(
        post_leave,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": {
          "Leave": {
            "id": "00000000-0000-0000-0000-000000000000"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::Leave(PostId::NULL))
        )
    );

    parse_case!(
        post_update_post,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": {
          "UpdatePost": [
            {
              "id": "00000000-0000-0000-0000-000000000000"
            },
            {
              "title": "Basics",
              "content": "Looking for practice partners.",
              "level": "Beginner1",
              "owner": {
                "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
              },
              "date": "2025-01-02T03:04:05Z",
              "partaking_users": [
                {
                  "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
                }
              ]
            }
          ]
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::UpdatePost(
                PostId::NULL,
                sample_post(UserId::test())
            ))
        )
    );

    parse_case!(
        post_new_post,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": {
          "NewPost": {
            "title": "Basics",
            "content": "Looking for practice partners.",
            "level": "Beginner1",
            "owner": {
              "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
            },
            "date": "2025-01-02T03:04:05Z",
            "partaking_users": [
              {
                "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
              }
            ]
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::NewPost(sample_post(UserId::test())))
        )
    );

    parse_case!(
        post_delete_post,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Post": {
          "DeletePost": {
            "id": "00000000-0000-0000-0000-000000000000"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Post(PostAction::DeletePost(PostId::NULL))
        )
    );

    parse_case!(
        chat_get_chat_for,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Chat": {
          "GetChatFor": {
            "id": "00000000-0000-0000-0000-000000000000"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Chat(ChatAction::GetChatFor(PostId::NULL))
        )
    );

    parse_case!(
        chat_get_chat,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Chat": {
          "GetChat": {
            "id": "01234567-89ab-cdef-0123-456789abcdef"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Chat(ChatAction::GetChat(sample_chat_id()))
        )
    );

    parse_case!(
        chat_send_message,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Chat": {
          "SendMessage": {
            "sender": {
              "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
            },
            "kind": {
              "Text": "Hello from chat."
            },
            "chat_id": {
              "id": "01234567-89ab-cdef-0123-456789abcdef"
            }
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2026_02_07,
            ClientToServer::Chat(ChatAction::SendMessage(sample_chat_message(
                UserId::test(),
                sample_chat_id()
            )))
        )
    );
}

mod invalid_parsing {
    use super::*;

    parse_case!(
        missing_version,
        r#"{
      "data": {
        "Post": "GetPosts"
      }
    }"#,
        ExpectedResult::Err
    );

    parse_case!(
        unknown_version,
        r#"{
      "version": "V2099_01_01",
      "data": "Hello"
    }"#,
        ExpectedResult::Err
    );

    parse_case!(
        invalid_chat_id,
        r#"{
      "version": "V2026_02_07",
      "data": {
        "Chat": {
          "GetChat": {
            "id": "invalid-chat-id"
          }
        }
      }
    }"#,
        ExpectedResult::Err
    );
}
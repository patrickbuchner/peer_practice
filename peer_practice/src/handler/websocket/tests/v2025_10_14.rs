use super::*;

mod valid_parsing {
    use super::*;
    use peer_practice_messages::current::messages::client_to_server::{PostAction, UserAction};
    use peer_practice_messages::current::post::PostId;
    
    parse_case!(
        message_not_yet_known,
        r#"{
      "version": "V2025_10_14",
      "data": "MessageNotYetKnown"
    }"#,
        ExpectedResult::Ok(Version::V2025_10_14, ClientToServer::MessageNotYetKnown)
    );

    parse_case!(
        hello,
        r#"{
      "version": "V2025_10_14",
      "data": "Hello"
    }"#,
        ExpectedResult::Ok(Version::V2025_10_14, ClientToServer::Hello)
    );

    parse_case!(
        get_user,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "GetUser": {
          "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::User(UserAction::Get(UserId::test()))
        )
    );

    parse_case!(
        update_user,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "UpdateUser": {
          "display_name": "Alice",
          "id": {
            "id": "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"
          }
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::User(UserAction::Update(sample_user_display(UserId::test())))
        )
    );

    parse_case!(
        get_posts,
        r#"{
      "version": "V2025_10_14",
      "data": "GetPosts"
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::GetPosts)
        )
    );

    parse_case!(
        join,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "Join": {
          "id": "00000000-0000-0000-0000-000000000000"
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::Join(PostId::NULL))
        )
    );

    parse_case!(
        leave,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "Leave": {
          "id": "00000000-0000-0000-0000-000000000000"
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::Leave(PostId::NULL))
        )
    );

    parse_case!(
        update_post,
        r#"{
      "version": "V2025_10_14",
      "data": {
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
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::UpdatePost(
                PostId::NULL,
                sample_post(UserId::test())
            ))
        )
    );

    parse_case!(
        new_post,
        r#"{
      "version": "V2025_10_14",
      "data": {
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
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::NewPost(sample_post(UserId::test())))
        )
    );

    parse_case!(
        delete_post,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "DeletePost": {
          "id": "00000000-0000-0000-0000-000000000000"
        }
      }
    }"#,
        ExpectedResult::Ok(
            Version::V2025_10_14,
            ClientToServer::Post(PostAction::DeletePost(PostId::NULL))
        )
    );
}

mod invalid_parsing {
    use super::*;

    parse_case!(
        missing_version,
        r#"{
      "data": "Hello"
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
        invalid_uuid_in_get_user,
        r#"{
      "version": "V2025_10_14",
      "data": {
        "GetUser": {
          "id": "not-a-uuid"
        }
      }
    }"#,
        ExpectedResult::Err
    );
}

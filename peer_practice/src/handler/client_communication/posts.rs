use crate::app_state::AppState;
use eyre::WrapErr;
use peer_practice_messages::current::messages::client_to_server::PostAction;
use peer_practice_messages::current::messages::{ServerToClient, server_to_client};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::posts::PostsMsg;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tokio::sync::oneshot;

pub async fn post_handler(
    action: PostAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) -> eyre::Result<()> {
    match action {
        PostAction::GetPosts => {
            let (ptx, prx) = oneshot::channel();
            state
                .posts
                .send(PostsMsg::List(ptx))
                .await
                .expect("Failed to get posts");
            if let Ok(posts) = prx.await {
                for (post_id, post) in posts {
                    state
                        .ws_hub
                        .send(WsHubMsg::Send {
                            user_id,
                            con_id,
                            msg: ServerToClient::Post(server_to_client::PostAction::Post(
                                post_id, post,
                            )),
                        })
                        .await
                        .wrap_err("Failed to send post to client")?;
                }
            }
        }
        PostAction::Join(post) => {
            state
                .posts
                .send(PostsMsg::UserJoins(post, user_id))
                .await
                .wrap_err("Failed to join post")?;
        }
        PostAction::Leave(post) => {
            state
                .posts
                .send(PostsMsg::UserLeaves(post, user_id))
                .await
                .wrap_err("Failed to leave post")?;
        }
        PostAction::UpdatePost(id, post) => {
            if post.owner == user_id {
                state
                    .posts
                    .send(PostsMsg::Upsert(id, post))
                    .await
                    .wrap_err("Failed to update post")?;
            }
        }
        PostAction::NewPost(mut post) => {
            post.owner = user_id;
            post.partaking_users.insert(user_id);
            let (tx, rx) = oneshot::channel();
            state
                .posts
                .send(PostsMsg::New(post, tx))
                .await
                .wrap_err("Failed to insert new post")?;
            rx.await.wrap_err("Failed to insert new post")?;
        }
        PostAction::DeletePost(post_id) => {
            let (tx, rx) = oneshot::channel();
            state
                .posts
                .send(PostsMsg::Get(post_id, tx))
                .await
                .wrap_err("Failed to get post")?;
            if let Ok(Some(post)) = rx.await
                && post.owner == user_id
            {
                state
                    .posts
                    .send(PostsMsg::Remove(post_id))
                    .await
                    .wrap_err("Failed to remove post")?;
            }
        }
        PostAction::GetPostMessages(_) => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::test_state;
    use peer_practice_messages::current::level::Level;
    use peer_practice_messages::current::post::{Post, PostId, Topics};
    use peer_practice_messages::test_helpers_impl::{
        expect_no_message, fixed_timestamp, recv_timeout,
    };
    use peer_practice_server_services::ws_hub::ConnectionId;
    use std::collections::HashSet;

    fn sample_post(owner: UserId) -> Post {
        Post {
            title: Topics::default(),
            content: "test".to_string(),
            level: Level::Beginner1,
            owner,
            date: fixed_timestamp(),
            partaking_users: HashSet::new(),
        }
    }

    #[tokio::test]
    async fn update_post_ignores_non_owner() {
        let (state, mut rx) = test_state();
        let owner = UserId::new();
        let other = UserId::new();
        let post = sample_post(owner);

        post_handler(
            PostAction::UpdatePost(PostId::new(), post),
            &state,
            other,
            ConnectionId::new(),
        )
        .await
        .expect("handler ok");

        expect_no_message(&mut rx.posts).await;
    }

    #[tokio::test]
    async fn delete_post_ignores_non_owner() {
        let (state, mut rx) = test_state();
        let owner = UserId::new();
        let other = UserId::new();
        let post_id = PostId::new();
        let post = sample_post(owner);

        let state = state.clone();
        let handler = tokio::spawn(async move {
            post_handler(
                PostAction::DeletePost(post_id),
                &state,
                other,
                ConnectionId::new(),
            )
            .await
        });

        match recv_timeout(&mut rx.posts).await {
            PostsMsg::Get(id, respond_to) => {
                assert_eq!(post_id, id);
                let _ = respond_to.send(Some(post));
            }
            _ => panic!("expected PostsMsg::Get"),
        }

        handler.await.expect("handler task ok").expect("handler ok");
        expect_no_message(&mut rx.posts).await;
    }
}

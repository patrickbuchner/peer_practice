use axum::extract::ws::{Message, WebSocket};
use tokio::sync::oneshot;
use tracing::{error, info};

use crate::app_state::AppState;
use peer_practice_server_services::posts::PostsMsg;
use peer_practice_server_services::users::UsersMsg;
use peer_practice_shared::messages::{ClientToServer, ServerToClient};
use peer_practice_shared::user::UserId;

pub async fn handle_websocket_message(
    socket: &mut WebSocket,
    state: &AppState,
    user_id: UserId,
    msg: ClientToServer,
) {
    match msg {
        ClientToServer::GetUser(user) => {
            info!(
                user_id = ?user_id,
                target_user_id = ?user,
                command = "GetUser",
                "received client command"
            );
            let (tx, rx) = oneshot::channel();
            _ = state
                .users
                .send(UsersMsg::GetById {
                    id: user,
                    respond_to: tx,
                })
                .await;
            if let Ok(user) = rx.await
                && let Some(user) = &user
            {
                match socket
                    .send(Message::Text(
                        serde_json::to_string(&ServerToClient::User(user.id, user.into()))
                            .unwrap()
                            .into(),
                    ))
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Error sending user: {:?}", err);
                    }
                }
            }
        }
        ClientToServer::UpdateUser(user_display) => {
            info!(
                user_id = ?user_id,
                target_user_id = ?user_display.id,
                display_name = ?user_display.display_name,
                command = "UpdateUser",
                "received client command"
            );
            if user_display.id == user_id {
                let (tx, rx) = oneshot::channel();
                _ = state
                    .users
                    .send(UsersMsg::GetById {
                        id: user_id,
                        respond_to: tx,
                    })
                    .await;
                if let Ok(user) = rx.await
                    && let Some(user) = &user
                {
                    let mut user = user.clone();
                    user.display_name = user_display.display_name;
                    _ = state
                        .users
                        .send(UsersMsg::Update { id: user_id, user })
                        .await;
                }
            }
        }
        ClientToServer::GetPosts => {
            info!(user_id = ?user_id, command = "GetPosts", "received client command");
            let (ptx, prx) = oneshot::channel();
            _ = state.posts.send(PostsMsg::List(ptx)).await;
            if let Ok(posts) = prx.await {
                for (post_id, post) in posts {
                    if socket
                        .send(Message::Text(
                            serde_json::to_string(&ServerToClient::Post(post_id, post))
                                .unwrap()
                                .into(),
                        ))
                        .await
                        .is_err()
                    {}
                }
            }
        }
        ClientToServer::Join(post) => {
            info!(user_id = ?user_id, post_id = ?post, command = "Join", "received client command");
            _ = state.posts.send(PostsMsg::UserJoins(post, user_id)).await;
        }
        ClientToServer::Leave(post) => {
            info!(user_id = ?user_id, post_id = ?post, command = "Leave", "received client command");
            _ = state.posts.send(PostsMsg::UserLeaves(post, user_id)).await;
        }
        ClientToServer::UpdatePost(id, post) => {
            info!(
                user_id = ?user_id,
                post_id = ?id,
                owner_id = ?post.owner,
                partaking_users = post.partaking_users.len(),
                command = "UpdatePost",
                "received client command"
            );
            if post.owner == user_id {
                _ = state.posts.send(PostsMsg::Upsert(id, post)).await;
            }
        }
        ClientToServer::NewPost(mut post) => {
            info!(
                user_id = ?user_id,
                owner_id = ?post.owner,
                partaking_users = post.partaking_users.len(),
                command = "NewPost",
                "received client command"
            );
            post.owner = user_id;
            post.partaking_users.insert(user_id);
            let (tx, rx) = oneshot::channel();
            _ = state.posts.send(PostsMsg::New(post, tx)).await;
            _ = rx.await;
        }
        ClientToServer::DeletePost(post_id) => {
            info!(
                user_id = ?user_id,
                post_id = ?post_id,
                command = "DeletePost",
                "received client command"
            );
            let (tx, rx) = oneshot::channel();
            _ = state.posts.send(PostsMsg::Get(post_id, tx)).await;
            if let Ok(Some(post)) = rx.await
                && post.owner == user_id
            {
                _ = state.posts.send(PostsMsg::Remove(post_id)).await;
            }
        }
    }
}

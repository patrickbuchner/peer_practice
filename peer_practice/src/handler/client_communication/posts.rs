use crate::app_state::AppState;
use peer_practice_server_services::posts::PostsMsg;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use peer_practice_shared::messages::client_to_server::PostAction;
use peer_practice_shared::messages::{ServerToClient, server_to_client};
use peer_practice_shared::user::UserId;
use tokio::sync::oneshot;
use tracing::info;

pub async fn post_handler(
    action: PostAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) {
    match action {
        PostAction::GetPosts => {
            info!(user_id = ?user_id, command = "GetPosts", "received client command");
            let (ptx, prx) = oneshot::channel();
            _ = state.posts.send(PostsMsg::List(ptx)).await;
            if let Ok(posts) = prx.await {
                for (post_id, post) in posts {
                    _ = state.ws_hub.send(WsHubMsg::Send {
                        user_id,
                        con_id,
                        msg: ServerToClient::Post(server_to_client::PostAction::Post(
                            post_id, post,
                        )),
                    });
                }
            }
        }
        PostAction::Join(post) => {
            info!(user_id = ?user_id, post_id = ?post, command = "Join", "received client command");
            _ = state.posts.send(PostsMsg::UserJoins(post, user_id)).await;
        }
        PostAction::Leave(post) => {
            info!(user_id = ?user_id, post_id = ?post, command = "Leave", "received client command");
            _ = state.posts.send(PostsMsg::UserLeaves(post, user_id)).await;
        }
        PostAction::UpdatePost(id, post) => {
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
        PostAction::NewPost(mut post) => {
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
        PostAction::DeletePost(post_id) => {
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
        PostAction::GetPostMessages(_) => {}
    }
}

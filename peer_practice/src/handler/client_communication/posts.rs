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

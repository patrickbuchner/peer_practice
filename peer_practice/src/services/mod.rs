use crate::app_state::AppState;

use chrono::{DateTime, Duration, Utc};
use peer_practice_server_services::posts::PostsMsg;

pub async fn remove_expired_posts(app_state: &AppState, now: DateTime<Utc>) -> eyre::Result<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app_state.posts.send(PostsMsg::List(tx)).await?;
    for (id, post) in rx.await? {
        let due = post.date + Duration::days(2);
        if due < now {
            app_state.posts.send(PostsMsg::Remove(id)).await?;
        }
    }

    Ok(())
}

pub async fn run_expired_posts_reaper(app_state: AppState, interval: Duration) {
    loop {
        let now = Utc::now();
        if let Err(err) = remove_expired_posts(&app_state, now).await {
            eprintln!("expired posts reaper error: {err}");
        }
        tokio::time::sleep(interval.to_std().unwrap()).await;
    }
}

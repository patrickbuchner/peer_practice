use crate::app_state::AppState;
use crate::task;
use chrono::{DateTime, Duration, Utc};
use peer_practice_server_services::active_sessions::ActiveSessionsMsg;
use peer_practice_server_services::posts::PostsMsg;

pub fn spawn_clean_up_services(state: &AppState) {
    task::spawn_named(
        "expired-posts-reaper",
        run_expired_posts_reaper(state.clone(), Duration::hours(1)),
    );

    task::spawn_named(
        "obsolete-jwt-remover",
        run_obsolete_jwt_remover(state.clone(), Duration::hours(1)),
    );
}

async fn remove_expired_posts(app_state: &AppState, now: DateTime<Utc>) -> eyre::Result<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app_state.posts.send(PostsMsg::List(tx)).await?;
    for (id, post) in rx.await? {
        let due = post.date + Duration::days(8);
        if due < now {
            app_state.posts.send(PostsMsg::Remove(id)).await?;
        }
    }

    Ok(())
}

async fn run_expired_posts_reaper(app_state: AppState, interval: Duration) {
    loop {
        let now = Utc::now();
        if let Err(err) = remove_expired_posts(&app_state, now).await {
            eprintln!("expired posts reaper error: {err}");
        }
        tokio::time::sleep(interval.to_std().unwrap()).await;
    }
}

async fn run_obsolete_jwt_remover(app_state: AppState, interval: Duration) {
    loop {
        _ = app_state
            .active_sessions
            .send(ActiveSessionsMsg::RemoveObsoleteJwts)
            .await;
        tokio::time::sleep(interval.to_std().unwrap()).await;
    }
}

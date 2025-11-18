use peer_practice_messages::Envelope;
use peer_practice_messages::current::post::{Post, PostId};
use peer_practice_messages::current::user::{User, UserId};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, trace};

#[derive(Debug)]
pub enum StorageMsg {
    SavePosts(HashMap<PostId, Post>),
    RetrievePosts {
        respond_to: oneshot::Sender<HashMap<PostId, Post>>,
    },
    SaveUsers(HashMap<UserId, User>),
    RetrieveUsers {
        respond_to: oneshot::Sender<HashMap<UserId, User>>,
    },
}

async fn save_snapshot(namespace: &str, data: &Value, work_dir: &Path) {
    let path = to_file_path(work_dir, namespace);
    match write_atomic_json(&path, data).await {
        Ok(()) => trace!("Saved snapshot '{}'", namespace),
        Err(err) => error!("SaveSnapshot '{}' failed: {}", namespace, err),
    }
}

async fn load_snapshot(namespace: &str, work_dir: &Path) -> Value {
    let path = to_file_path(work_dir, namespace);
    match read_json(&path).await {
        Ok(val) => val,
        Err(err) => {
            info!("LoadSnapshot '{}' defaulting to null: {}", namespace, err);
            Value::Null
        }
    }
}

pub fn spawn_storage_actor(work_dir: PathBuf) -> mpsc::Sender<StorageMsg> {
    let (tx, mut rx) = mpsc::channel::<StorageMsg>(128);

    tokio::spawn(async move {
        if let Err(err) = fs::create_dir_all(&work_dir).await {
            error!("Failed to create work_dir {:?}: {}", work_dir, err);
        }

        while let Some(msg) = rx.recv().await {
            match msg {
                StorageMsg::SavePosts(posts) => {
                    let pairs = posts
                        .iter()
                        .map(|(id, post)| json!([id, post]))
                        .collect::<Vec<_>>();
                    save_snapshot("posts", &Value::Array(pairs), &work_dir).await;
                }
                StorageMsg::RetrievePosts { respond_to } => {
                    let mut posts = HashMap::new();
                    let value = load_snapshot("posts", &work_dir).await;
                    if let Value::Array(entries) = value {
                        for entry in entries {
                            if let Value::Array(mut pair) = entry
                                && pair.len() == 2
                                && let (Ok(id), Ok(post)) = (
                                    serde_json::from_value::<PostId>(pair.remove(0)),
                                    serde_json::from_value::<Post>(pair.remove(0)),
                                )
                            {
                                posts.insert(id, post.clone());
                            }
                        }
                    }

                    let _ = respond_to.send(posts);
                }
                StorageMsg::SaveUsers(users) => {
                    let pairs = users
                        .iter()
                        .map(|(id, user)| json!([id, user]))
                        .collect::<Vec<_>>();
                    save_snapshot("users", &Value::Array(pairs), &work_dir).await;
                }
                StorageMsg::RetrieveUsers { respond_to } => {
                    let mut users = HashMap::new();
                    let value = load_snapshot("users", &work_dir).await;
                    info!("Retrieved users: {:?}", value);
                    if let Value::Array(entries) = value {
                        for entry in entries {
                            if let Value::Array(mut pair) = entry
                                && pair.len() == 2
                                && let (Ok(id), Ok(post)) = (
                                    serde_json::from_value::<UserId>(pair.remove(0)),
                                    serde_json::from_value::<User>(pair.remove(0)),
                                )
                            {
                                users.insert(id, post.clone());
                            }
                        }
                    }

                    let _ = respond_to.send(users);
                }
            }
        }
    });

    tx
}

fn to_file_path(work_dir: &Path, namespace: &str) -> PathBuf {
    let cleaned = namespace
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect::<String>();
    work_dir.join(format!("{cleaned}.json"))
}

async fn write_atomic_json(path: &Path, value: &Value) -> eyre::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }

    let data = Envelope {
        version: peer_practice_messages::Version::V2025_10_14,
        data: value,
    };
    let data = serde_json::to_vec_pretty(&data)?;

    let tmp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&tmp).await?;
    file.write_all(&data).await?;
    file.flush().await?;
    fs::rename(&tmp, path).await?;
    Ok(())
}

async fn read_json<T: DeserializeOwned>(path: &Path) -> eyre::Result<T> {
    let data = fs::read(path).await?;
    if let Ok(enveloped) = serde_json::from_slice::<Envelope<T>>(&data) {
        Ok(enveloped.data)
    } else {
        let value = serde_json::from_slice::<T>(&data)?;
        Ok(value)
    }
}

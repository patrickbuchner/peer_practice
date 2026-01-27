use crate::chat::progress::Progress;
use futures_util::future::BoxFuture;
use peer_practice_messages::Envelope;
use peer_practice_messages::current::chat::ChatId;
use peer_practice_messages::current::post::{Post, PostId};
use peer_practice_messages::current::user::{User, UserId};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;
use tracing::{error, info, trace};

#[cfg(test)]
mod test;

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
    SaveChats(HashMap<ChatId, Progress>),
    RetrieveChats {
        respond_to: oneshot::Sender<HashMap<ChatId, Progress>>,
    },
    Ping {
        respond_to: oneshot::Sender<()>,
    },
}

trait StorageFs: Send + Sync + 'static {
    fn create_dir_all(&self, path: PathBuf) -> BoxFuture<'static, io::Result<()>>;
    fn read(&self, path: PathBuf) -> BoxFuture<'static, io::Result<Vec<u8>>>;
    fn write(&self, path: PathBuf, data: Vec<u8>) -> BoxFuture<'static, io::Result<()>>;
    fn rename(&self, from: PathBuf, to: PathBuf) -> BoxFuture<'static, io::Result<()>>;
}

#[derive(Clone, Copy, Debug)]
struct TokioFs;

impl StorageFs for TokioFs {
    fn create_dir_all(&self, path: PathBuf) -> BoxFuture<'static, io::Result<()>> {
        Box::pin(async move { fs::create_dir_all(path).await })
    }

    fn read(&self, path: PathBuf) -> BoxFuture<'static, io::Result<Vec<u8>>> {
        Box::pin(async move { fs::read(path).await })
    }

    fn write(&self, path: PathBuf, data: Vec<u8>) -> BoxFuture<'static, io::Result<()>> {
        Box::pin(async move { fs::write(path, data).await })
    }

    fn rename(&self, from: PathBuf, to: PathBuf) -> BoxFuture<'static, io::Result<()>> {
        Box::pin(async move { fs::rename(from, to).await })
    }
}

async fn save_snapshot(fs: &dyn StorageFs, namespace: &str, data: &Value, work_dir: &Path) {
    let path = to_file_path(work_dir, namespace);
    match write_atomic_json(fs, &path, data).await {
        Ok(()) => trace!("Saved snapshot '{}'", namespace),
        Err(err) => error!("SaveSnapshot '{}' failed: {}", namespace, err),
    }
}

async fn load_snapshot(fs: &dyn StorageFs, namespace: &str, work_dir: &Path) -> Value {
    let path = to_file_path(work_dir, namespace);
    match read_json(fs, &path).await {
        Ok(val) => val,
        Err(err) => {
            info!("LoadSnapshot '{}' defaulting to null: {}", namespace, err);
            Value::Null
        }
    }
}

pub async fn handle_storage_operations(work_dir: PathBuf, rx: Receiver<StorageMsg>) {
    handle_storage_operations_with_fs(work_dir, TokioFs, rx).await
}

async fn handle_storage_operations_with_fs(
    work_dir: PathBuf,
    fs: impl StorageFs,
    mut rx: Receiver<StorageMsg>,
) {
    if let Err(err) = fs.create_dir_all(work_dir.clone()).await {
        error!("Failed to create work_dir {:?}: {}", work_dir, err);
    }

    while let Some(msg) = rx.recv().await {
        match msg {
            StorageMsg::SavePosts(posts) => {
                let pairs = posts
                    .iter()
                    .map(|(id, post)| json!([id, post]))
                    .collect::<Vec<_>>();
                save_snapshot(&fs, "posts", &Value::Array(pairs), &work_dir).await;
            }
            StorageMsg::RetrievePosts { respond_to } => {
                let mut posts = HashMap::new();
                let value = load_snapshot(&fs, "posts", &work_dir).await;
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
                save_snapshot(&fs, "users", &Value::Array(pairs), &work_dir).await;
            }
            StorageMsg::RetrieveUsers { respond_to } => {
                let mut users = HashMap::new();
                let value = load_snapshot(&fs, "users", &work_dir).await;
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
            StorageMsg::SaveChats(chats) => {
                let pairs = chats
                    .iter()
                    .map(|(id, chat)| json!([id, chat]))
                    .collect::<Vec<_>>();
                save_snapshot(&fs, "chats", &Value::Array(pairs), &work_dir).await;
            }
            StorageMsg::RetrieveChats { respond_to } => {
                let mut chats = HashMap::new();
                let value = load_snapshot(&fs, "chats", &work_dir).await;
                if let Value::Array(entries) = value {
                    for entry in entries {
                        if let Value::Array(mut pair) = entry
                            && pair.len() == 2
                            && let (Ok(id), Ok(chat)) = (
                                serde_json::from_value::<ChatId>(pair.remove(0)),
                                serde_json::from_value::<Progress>(pair.remove(0)),
                            )
                        {
                            chats.insert(id, chat.clone());
                        }
                    }
                }

                let _ = respond_to.send(chats);
            }
            StorageMsg::Ping { respond_to } => {
                let _ = respond_to.send(());
            }
        }
    }
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

async fn write_atomic_json(fs: &dyn StorageFs, path: &Path, value: &Value) -> eyre::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs.create_dir_all(parent.to_path_buf()).await;
    }

    let data = Envelope {
        version: peer_practice_messages::Version::V2025_10_14,
        data: value,
    };
    let data = serde_json::to_vec_pretty(&data)?;

    let tmp = path.with_extension("json.tmp");
    fs.write(tmp.clone(), data)
        .await
        .map_err(|e| eyre::eyre!(e))?;
    fs.rename(tmp, path.to_path_buf())
        .await
        .map_err(|e| eyre::eyre!(e))?;
    Ok(())
}

async fn read_json<T: DeserializeOwned>(fs: &dyn StorageFs, path: &Path) -> eyre::Result<T> {
    let data = fs
        .read(path.to_path_buf())
        .await
        .map_err(|e| eyre::eyre!(e))?;
    if let Ok(enveloped) = serde_json::from_slice::<Envelope<T>>(&data) {
        Ok(enveloped.data)
    } else {
        let value = serde_json::from_slice::<T>(&data)?;
        Ok(value)
    }
}

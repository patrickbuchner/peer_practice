use super::*;
use crate::chat::message::Message;
use peer_practice_messages::current::email::Email;
use peer_practice_messages::current::level::Level;
use peer_practice_messages::current::post::Topics;
use peer_practice_messages::test_helpers_impl::fixed_timestamp;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Default)]
struct MemFs {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

impl StorageFs for MemFs {
    fn create_dir_all(&self, _path: PathBuf) -> BoxFuture<'static, io::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn read(&self, path: PathBuf) -> BoxFuture<'static, io::Result<Vec<u8>>> {
        let files = self.files.clone();
        Box::pin(async move {
            files
                .lock()
                .unwrap()
                .get(&path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing file"))
        })
    }

    fn write(&self, path: PathBuf, data: Vec<u8>) -> BoxFuture<'static, io::Result<()>> {
        let files = self.files.clone();
        Box::pin(async move {
            files.lock().unwrap().insert(path, data);
            Ok(())
        })
    }

    fn rename(&self, from: PathBuf, to: PathBuf) -> BoxFuture<'static, io::Result<()>> {
        let files = self.files.clone();
        Box::pin(async move {
            let mut guard = files.lock().unwrap();
            let data = guard
                .remove(&from)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing source"))?;
            guard.insert(to, data);
            Ok(())
        })
    }
}

async fn arrange(
    fs: MemFs,
) -> (
    mpsc::Sender<StorageMsg>,
    tokio::task::JoinHandle<()>,
    PathBuf,
) {
    let (tx, rx) = mpsc::channel::<StorageMsg>(16);
    let work_dir = PathBuf::from("/mem");
    let task = tokio::spawn(handle_storage_operations_with_fs(work_dir.clone(), fs, rx));
    (tx, task, work_dir)
}

async fn recv_oneshot<T>(rx: oneshot::Receiver<T>) -> T {
    rx.await.expect("oneshot closed")
}

async fn ping(tx: &mpsc::Sender<StorageMsg>) {
    let (respond_to, recv) = oneshot::channel();
    tx.send(StorageMsg::Ping { respond_to })
        .await
        .unwrap();
    recv_oneshot(recv).await
}

async fn retrieve_posts(tx: &mpsc::Sender<StorageMsg>) -> HashMap<PostId, Post> {
    let (respond_to, recv) = oneshot::channel();
    tx.send(StorageMsg::RetrievePosts { respond_to })
        .await
        .unwrap();
    recv_oneshot(recv).await
}

async fn retrieve_users(tx: &mpsc::Sender<StorageMsg>) -> HashMap<UserId, User> {
    let (respond_to, recv) = oneshot::channel();
    tx.send(StorageMsg::RetrieveUsers { respond_to })
        .await
        .unwrap();
    recv_oneshot(recv).await
}

async fn retrieve_chats(tx: &mpsc::Sender<StorageMsg>) -> HashMap<ChatId, Progress> {
    let (respond_to, recv) = oneshot::channel();
    tx.send(StorageMsg::RetrieveChats { respond_to })
        .await
        .unwrap();
    recv_oneshot(recv).await
}

fn mk_post(owner: UserId) -> Post {
    Post {
        title: Topics::default(),
        content: "hello".to_string(),
        level: Level::Beginner1,
        owner,
        date: fixed_timestamp(),
        partaking_users: Default::default(),
    }
}

fn mk_user(id: UserId, email: &str) -> User {
    User {
        id,
        email: Email::new(email).unwrap(),
        display_name: Some("Tester".to_string()),
    }
}

fn mk_progress(chat_id: ChatId, post_id: PostId) -> Progress {
    Progress {
        chat_id,
        post_id,
        content: vec![Message {
            sender: UserId::default(),
            message: "hi".to_string(),
            chat_id,
            timestamp: fixed_timestamp(),
        }],
    }
}

#[tokio::test]
async fn posts_roundtrip_save_then_retrieve() {
    // Arrange
    let fs = MemFs::default();
    let (tx, task, _work_dir) = arrange(fs).await;
    ping(&tx).await;

    let mut posts = HashMap::new();
    let id = PostId::new();
    posts.insert(id, mk_post(UserId::default()));

    // Act
    tx.send(StorageMsg::SavePosts(posts.clone())).await.unwrap();
    let got = retrieve_posts(&tx).await;

    // Assert
    assert_eq!(posts.len(), got.len());
    assert_eq!(
        posts.get(&id).unwrap().content,
        got.get(&id).unwrap().content
    );

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn users_roundtrip_save_then_retrieve() {
    // Arrange
    let fs = MemFs::default();
    let (tx, task, _work_dir) = arrange(fs).await;
    ping(&tx).await;

    let mut users = HashMap::new();
    let id = UserId::new();
    users.insert(id, mk_user(id, "user@example.com"));

    // Act
    tx.send(StorageMsg::SaveUsers(users.clone())).await.unwrap();
    let got = retrieve_users(&tx).await;

    // Assert
    assert_eq!(users.len(), got.len());
    assert_eq!(
        users.get(&id).unwrap().email.value(),
        got.get(&id).unwrap().email.value()
    );

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn chats_roundtrip_save_then_retrieve() {
    // Arrange
    let fs = MemFs::default();
    let (tx, task, _work_dir) = arrange(fs).await;
    ping(&tx).await;

    let mut chats = HashMap::new();
    let chat_id = ChatId::new();
    let post_id = PostId::new();
    chats.insert(chat_id, mk_progress(chat_id, post_id));

    // Act
    tx.send(StorageMsg::SaveChats(chats.clone())).await.unwrap();
    let got = retrieve_chats(&tx).await;

    // Assert
    assert_eq!(chats.len(), got.len());
    assert_eq!(
        chats.get(&chat_id).unwrap().content.len(),
        got.get(&chat_id).unwrap().content.len()
    );

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn retrieve_defaults_to_empty_on_corrupted_json() {
    // Arrange
    let fs = MemFs::default();
    let (tx, task, work_dir) = arrange(fs.clone()).await;
    ping(&tx).await;

    let path = to_file_path(&work_dir, "posts");
    fs.write(path, b"{not valid json".to_vec()).await.unwrap();

    // Act
    let got = retrieve_posts(&tx).await;

    // Assert
    assert!(got.is_empty());

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn legacy_non_enveloped_json_is_supported() {
    // Arrange
    let fs = MemFs::default();
    let (tx, task, work_dir) = arrange(fs.clone()).await;
    ping(&tx).await;

    let id = PostId::new();
    let post = mk_post(UserId::default());
    let legacy_value = Value::Array(vec![json!([id, post])]);
    let legacy_bytes = serde_json::to_vec_pretty(&legacy_value).unwrap();

    let path = to_file_path(&work_dir, "posts");
    fs.write(path, legacy_bytes).await.unwrap();

    // Act
    let got = retrieve_posts(&tx).await;

    // Assert
    assert_eq!(1, got.len());
    assert!(got.contains_key(&id));

    drop(tx);
    let _ = task.await;
}

#[test]
fn to_file_path_sanitizes_namespace() {
    // Arrange
    let work_dir = PathBuf::from("/mem");

    // Act
    let path = to_file_path(&work_dir, "posts/../bad name");

    // Assert
    assert_eq!(PathBuf::from("/mem/posts_.._bad_name.json"), path);
}

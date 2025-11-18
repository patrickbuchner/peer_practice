use chrono::NaiveDate;
use peer_practice_shared::level::Level;
use peer_practice_shared::post::PostId;
use serde::{Deserialize, Serialize};
use web_sys::window;

#[derive(Clone, Serialize, Deserialize)]
pub struct Draft {
    pub title: String,
    pub ideas: String,
    pub level: Level,
    pub date: NaiveDate,
}

pub fn storage_key(post_id: PostId) -> String {
    format!("event_edit_draft:{post_id}",)
}

pub fn load_draft(post_id: PostId) -> Option<Draft> {
    let storage = window().and_then(|w| w.local_storage().ok().flatten());
    let key = storage_key(post_id);
    let s = storage.as_ref()?.get_item(&key).ok().flatten()?;
    serde_json::from_str::<Draft>(&s).ok()
}

pub fn save_draft(post_id: PostId, draft: &Draft) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten())
        && let Ok(s) = serde_json::to_string(draft)
    {
        let _ = storage.set_item(&storage_key(post_id), &s);
    }
}

pub fn clear_draft(post_id: PostId) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item(&storage_key(post_id));
    }
}

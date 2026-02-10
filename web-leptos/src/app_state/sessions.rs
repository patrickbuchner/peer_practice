use leptos::prelude::{ReadSignal, WriteSignal, signal};
use peer_practice_shared::sessions::{SessionId, SessionInformation};
use std::collections::HashMap;

pub fn create_sessions() -> (SessionsReader, SessionsWriter) {
    let current = signal(None);
    let sessions = signal(HashMap::new());
    let reader = SessionsReader {
        current: current.0,
        sessions: sessions.0,
    };
    let writer = SessionsWriter {
        current: current.1,
        sessions: sessions.1,
    };
    (reader, writer)
}

#[derive(Copy, Clone)]
pub struct SessionsWriter {
    pub current: WriteSignal<Option<SessionId>>,
    pub sessions: WriteSignal<HashMap<SessionId, SessionInformation>>,
}

#[derive(Copy, Clone)]
pub struct SessionsReader {
    pub current: ReadSignal<Option<SessionId>>,
    pub sessions: ReadSignal<HashMap<SessionId, SessionInformation>>,
}

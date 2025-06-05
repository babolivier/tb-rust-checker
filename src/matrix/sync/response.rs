use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::matrix::MessageEventContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SyncResponse {
    pub rooms: Option<Rooms>,
    pub next_batch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Rooms {
    pub join: HashMap<String, JoinedRoom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct JoinedRoom {
    pub timeline: RoomTimeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RoomTimeline {
    pub events: Vec<MessageEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct MessageEvent {
    pub content: MessageEventContent,
}

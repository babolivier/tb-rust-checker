use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(super) struct RoomEventFilter {
    pub event_fields: Vec<String>,
    pub room: RoomFilter,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RoomFilter {
    pub timeline: TimelineFilter,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TimelineFilter {
    pub rooms: Vec<String>,
    pub types: Vec<String>,
}

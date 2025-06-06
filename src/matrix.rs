use serde::{Deserialize, Serialize};

pub(crate) use sync::sync;

mod send;
mod sync;

/// The `msgtype` property of an `m.room.message` Matrix event content.
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    #[serde(rename = "m.text")]
    Text,
    #[serde(rename = "m.notice")]
    Notice,
    #[serde(rename = "m.key.verification.request")]
    KeyVerificationRequest,
    #[serde(rename = "m.image")]
    Image,
    #[serde(rename = "m.file")]
    File,
    #[serde(rename = "m.audio")]
    Audio,
    #[serde(rename = "m.video")]
    Video,
}

/// The content of an `m.room.message` Matrix event.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageEventContent {
    pub body: Option<String>,
    pub msgtype: Option<MessageType>,
}

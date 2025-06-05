use serde::{Deserialize, Serialize};

pub(crate) use sync::sync;

mod send;
mod sync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(self) enum MessageType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(self) struct MessageEventContent {
    pub body: Option<String>,
    pub msgtype: Option<MessageType>,
}

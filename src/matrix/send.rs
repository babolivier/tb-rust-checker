use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Error;
use reqwest::Client;
use url::Url;

use crate::{
    config::MatrixConfig,
    matrix::{MessageEventContent, MessageType},
};

pub(crate) async fn send_notice(
    matrix_cfg: MatrixConfig,
    client: Client,
    message: String,
) -> Result<(), Error> {
    let content = MessageEventContent {
        body: Some(message),
        msgtype: Some(MessageType::Notice),
    };
    let content = serde_json::to_string(&content)?;

    // Use the current timestamp as the transaction ID.
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // We know the current time is after the UNIX epoch, so unwrapping
        // should not panic.
        .unwrap()
        .as_millis();

    let url = format!(
        "https://{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
        matrix_cfg.server_host, matrix_cfg.room_id, now_ts
    );
    let url = Url::parse(&url)?;

    client
        .put(url)
        .bearer_auth(matrix_cfg.access_token)
        .body(content)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::Client;
use url::Url;

use crate::config::MatrixConfig;
use crate::error::Error;
use crate::matrix::{MessageEventContent, MessageType};

/// Send an `m.room.message` event to the target room specified in the
/// configuration.
///
/// The message's content has `m.notice` as its message type and the provided
/// text as the body.
pub(super) async fn send_notice(
    matrix_cfg: &MatrixConfig,
    client: Client,
    message: &str,
) -> Result<(), Error> {
    // Build and string-ified the event content.
    let content = MessageEventContent {
        body: Some(message.to_owned()),
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

    // Send the request, with HTTP errors propagated as Rust errors since no
    // non-2XX response is expected here.
    client
        .put(url)
        .bearer_auth(matrix_cfg.access_token.clone())
        .body(content)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

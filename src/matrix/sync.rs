use std::cell::RefCell;
use std::time::Duration;

use reqwest::Client;
use tokio::time;
use url::Url;

use crate::checksums::verify_checksums_match;
use crate::config::Config;
use crate::error::Error;
use crate::matrix::send::send_notice;
use crate::storage::store_sync_token;

use self::{
    filter::{EventFilter, RoomFilter, TimelineFilter},
    response::SyncResponse,
};

use super::MessageType;

mod filter;
mod response;

/// The amount of time to tell the server to wait for before responding if there
/// are no new messages since the last sync.
///
/// Setting this parameter avoids having to flood the server with requests in
/// this case, since the default timeout is `0` (as per the Matrix spec).
const SYNC_TIMEOUT: usize = 30000;

/// Send a sync request and process the response.
///
/// If the response includes messages for a push, download the relevant files
/// from mozilla-central and compare their checksums to the ones stored in
/// comm-central, then send the appropriate message to the Matrix room.
///
/// This happens only once per sync, even if it includes multiple push messages
/// (since we always compare with repository tips).
async fn do_sync(cfg: &Config, client: Client, url: Url) -> Result<String, Error> {
    // Send a new sync request and parse the sync response.
    let response: SyncResponse = client
        .get(url)
        .bearer_auth(&cfg.matrix.access_token)
        .send()
        .await?
        .json()
        .await?;

    let next_token = response.next_batch;

    // If there's no message for us to process, just bail early.
    let rooms = match response.rooms {
        Some(rooms) => rooms,
        None => {
            store_sync_token(cfg, &next_token).await?;
            return Ok(next_token);
        }
    };

    let events = match rooms.join.get(&cfg.matrix.room_id) {
        Some(room) => &room.timeline.events,
        None => {
            store_sync_token(cfg, &next_token).await?;
            return Ok(next_token);
        }
    };

    // Try to find at least one message matching a push in the sync response.
    let pushes = events
        .iter()
        // The event content might be empty if the event was redacted.
        // Because it's empty in this case, and not missing, serde won't
        // deserialize it as `Option::None`. We could solve this with a
        // custom implementation of `Deserialize` on `MessageEventContent`,
        // but it's easier, and correct in practice, to assume that when the
        // message type is present then the body is too.
        .filter_map(|event| match &event.content.msgtype {
            Some(MessageType::Notice) => Some(event.content.body.clone().unwrap()),
            _ => None,
        })
        .filter(|body| body.contains(&cfg.push_message_substring))
        .collect::<Vec<_>>();

    // If there's any push, then compare the checksums and send the appropriate
    // message.
    if !pushes.is_empty() {
        log::info!("Processing new push");

        if verify_checksums_match().await? {
            log::debug!("Checksums match");
            send_notice(&cfg.matrix, client.clone(), &cfg.messages.deps_up_to_date).await?;
        } else {
            log::debug!("Checksums do not match");
            send_notice(&cfg.matrix, client.clone(), &cfg.messages.deps_out_of_date).await?;
        }
    }

    store_sync_token(cfg, &next_token).await?;
    Ok(next_token)
}

/// Start a never-ending sync loop.
///
/// Each iteration of the loop sends a sync request and processes its response.
/// See the documentation for [`do_sync`] for more details.
///
/// If an I/O error occurs (when updating the stored sync token), this function
/// returns with it.
pub(crate) async fn sync(cfg: &Config, token: String) -> Result<(), Error> {
    // Filter sync responses for messages in the target room, and limit the
    // properties returned to the event content itself.
    let filter = EventFilter {
        event_fields: vec!["content".into()],
        room: RoomFilter {
            timeline: TimelineFilter {
                rooms: vec![cfg.matrix.room_id.clone()],
                types: vec!["m.room.message".into()],
            },
        },
    };

    // Build the base URL with the config and the string-ified filter. This URL
    // is essentially the full URL used for syncing, but without the token since
    // it will change for each iteration of the loop.
    let filter = serde_json::to_string(&filter)?;
    let base_url = format!(
        "https://{}/_matrix/client/v3/sync?filter={}&timeout={}",
        cfg.matrix.server_host, filter, SYNC_TIMEOUT,
    );
    let base_url = Url::parse(&base_url)?;

    // Loop indefinitely to listen for new messages in the room.
    let client = Client::new();
    let token = RefCell::new(token);
    loop {
        let mut url = base_url.clone();

        log::debug!("Syncing with token: {}", token.borrow());
        if !token.borrow().is_empty() {
            // We know there's always a query because we've defined one in the
            // base URL.
            let query = url.query().unwrap();
            let query = format!("{query}&since={}", token.borrow());
            url.set_query(Some(query.as_str()));
        }

        // Send the sync request and process the response.
        match do_sync(cfg, client.clone(), url).await {
            Ok(sync_token) => {
                token.replace(sync_token);
            }
            Err(err) => {
                match err {
                    // If an I/O error happened (e.g. if the file
                    // doesn't exist), it's unlikely to go away in a
                    // future iteration of the loop, so we want to exit
                    // here.
                    Error::Io(_) => return Err(err),
                    _ => {
                        // When any other error, log it and try again in 30s.
                        log::error!("Error while processing the last sync: {}", err);
                        time::sleep(Duration::from_secs(30)).await;
                    }
                }
            }
        }
    }
}

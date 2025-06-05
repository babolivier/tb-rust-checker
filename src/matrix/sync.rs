use std::cell::RefCell;

use anyhow::Error;
use reqwest::Client;
use tokio::signal;
use url::Url;

use crate::{checksums::compare_checksums, config::Config, matrix::send::send_notice};

use self::{
    filter::{RoomEventFilter, RoomFilter, TimelineFilter},
    response::SyncResponse,
};

use super::MessageType;

mod filter;
mod response;

async fn do_sync(cfg: Config, client: Client, url: Url) -> Result<String, Error> {
    let response: SyncResponse = client
        .get(url)
        .bearer_auth(&cfg.matrix.access_token)
        .send()
        .await?
        .json()
        .await?;

    let rooms = match response.rooms {
        Some(rooms) => rooms,
        None => return Ok(response.next_batch),
    };

    let events = match rooms.join.get(&cfg.matrix.room_id) {
        Some(room) => &room.timeline.events,
        None => return Ok(response.next_batch),
    };

    let pushes = events
        .into_iter()
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

    if pushes.len() > 0 {
        println!("new push!");
        if compare_checksums().await? {
            send_notice(
                cfg.matrix.clone(),
                client.clone(),
                cfg.messages.deps_up_to_date,
            )
            .await?;
        } else {
            send_notice(
                cfg.matrix.clone(),
                client.clone(),
                cfg.messages.deps_out_of_date,
            )
            .await?;
        }
    }

    Ok(response.next_batch)
}

pub(crate) async fn sync(cfg: Config, token: String) -> Result<(), Error> {
    let filter = RoomEventFilter {
        event_fields: vec!["content".into()],
        room: RoomFilter {
            timeline: TimelineFilter {
                rooms: vec![cfg.matrix.room_id.clone()],
                types: vec!["m.room.message".into()],
            },
        },
    };

    let filter = serde_json::to_string(&filter)?;
    let client = Client::new();

    let base_url = format!(
        "https://{}/_matrix/client/v3/sync?filter={}&timeout=30000",
        cfg.matrix.server_host, filter
    );
    let base_url = Url::parse(&base_url)?;
    let token = RefCell::new(token);
    loop {
        let mut url = base_url.clone();

        if !token.borrow().is_empty() {
            println!("syncing with token: {}", token.borrow());
            // We know there's always a query because we've defined one in the
            // base URL.
            let query = url.query().unwrap();
            let query = format!("{query}&since={}", token.borrow());
            url.set_query(Some(query.as_str()));
        }

        tokio::select! {
            result = do_sync(cfg.clone(), client.clone(), url) => {
                match result {
                    Ok(sync_token) => {
                        token.replace(sync_token);
                    },
                    Err(err) => eprintln!("error while talking to the Matrix server: {:?}", err),
                }
            },
            _ = signal::ctrl_c() => {
                println!("current sync token: {}", token.borrow());
                return Ok(());
            }
        }
    }
}

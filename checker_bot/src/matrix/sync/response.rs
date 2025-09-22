/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::matrix::MessageEventContent;

/// A response to a sync request.
///
/// This and all nested structs only include properties relevant to us.
///
/// See <https://spec.matrix.org/v1.14/client-server-api/#get_matrixclientv3sync>
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

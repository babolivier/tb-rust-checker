/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde::Serialize;

/// A stripped-down representation of an `EventFilter` used in sync requests.
///
/// This and all nested structs only include properties relevant for our sync
/// requests.
///
/// See <https://spec.matrix.org/v1.14/client-server-api/#post_matrixclientv3useruseridfilter_request_eventfilter>
#[derive(Debug, Clone, Serialize)]
pub(super) struct EventFilter {
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

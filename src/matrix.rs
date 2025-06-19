/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};

pub(crate) use sync::sync;

mod send;
mod sync;

/// The `msgtype` property of an `m.room.message` Matrix event content.
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    #[serde(rename = "m.notice")]
    Notice,

    // We only care about notices (because that's what we read and send). We
    // lump everything else in a catch-all variant so serde doesn't complain
    // about not being able to find one that fits.
    #[serde(other)]
    Unsupported,
}

/// The content of an `m.room.message` Matrix event.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageEventContent {
    pub body: Option<String>,
    pub msgtype: Option<MessageType>,
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io;

use thiserror::Error;

/// Errors that happen after the bot's initial setup.
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON (de)serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

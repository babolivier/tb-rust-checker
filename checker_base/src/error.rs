/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::error::Error as StdError;
use std::fmt::Display;
use std::io;

use thiserror::Error;

/// Errors that happen after the bot's initial setup.
#[derive(Error, Debug)]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    Io(#[from] io::Error),
    Json(#[from] serde_json::Error),
    UrlParse(#[from] url::ParseError),
}

impl Display for Error {
    // We don't implement `Display` using thiserror's `error` attributes,
    // because we need some extra processing to properly handle reqwest errors.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(error) => {
                write!(f, "network error: {error}")?;

                // If the error has a source, then we want to include it as
                // well, because reqwest's error itself might not include any
                // helpful data. For example, a deserialization error from serde
                // would be displayed as just "error decoding response body",
                // rather than anything indicating what's wrong with the
                // response body.
                if let Some(source) = error.source() {
                    write!(f, ": {source}")?;
                }

                Ok(())
            }
            Error::Io(error) => write!(f, "I/O error: {error}"),
            Error::Json(error) => write!(f, "JSON (de)serialization error: {error}"),
            Error::UrlParse(error) => write!(f, "URL parse error: {error}"),
        }
    }
}

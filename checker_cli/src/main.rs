/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;

use clap::Parser;

use checker_base::checksums::{ChangeSet, verify_checksums_match};

#[derive(Parser, Debug)]
struct Args {
    /// The mozilla-central revision to use. Defaults to "tip".
    #[arg(short, long)]
    mozilla_rev: Option<String>,

    /// The comm-central revision to use. Defaults to "tip".
    #[arg(short, long)]
    comm_rev: Option<String>,
}

impl From<Args> for ChangeSet {
    fn from(value: Args) -> Self {
        ChangeSet {
            moz_rev: value.mozilla_rev,
            tb_rev: value.comm_rev,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    env_logger::init();

    // Hack to set the default log level to "info".
    if env::var("RUST_LOG").is_err() {
        // SAFETY: We're always running this program in a single-thread context.
        unsafe { env::set_var("RUST_LOG", "info") }
    }

    match verify_checksums_match(args.into()).await {
        Ok(ok) => log::info!("checksums match: {ok}"),
        Err(err) => log::error!("error while verifying files: {err}"),
    }
}

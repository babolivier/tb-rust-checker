/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use clap::Parser;

use checker_base::checksums::{ChangeSet, verify_checksums_match};
use env_logger::Env;

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

    // Default the log level to "info".
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    match verify_checksums_match(args.into()).await {
        Ok(ok) => log::info!("checksums match: {ok}"),
        Err(err) => log::error!("error while verifying files: {err}"),
    }
}

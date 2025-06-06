use std::io::ErrorKind;

use clap::Parser;

use crate::config::load_config_from_file;
use crate::storage::read_sync_token_from_store;

mod checksums;
mod config;
mod error;
mod matrix;
mod storage;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_file: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    env_logger::init();

    let cfg = match load_config_from_file(args.config_file) {
        Ok(cfg) => cfg,
        Err(err) => panic!("error parsing config: {err:?}"),
    };

    log::info!("Parsed config");

    let sync_token = match read_sync_token_from_store(&cfg).await {
        Ok(token) => token,
        Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
        Err(err) => panic!("error reading sync token from storage: {err:?}"),
    };

    log::info!("Read stored sync token (if any)");

    match matrix::sync(&cfg, sync_token).await {
        Ok(_) => {}
        Err(err) => panic!("sync loop exited with error: {err:?}"),
    }
}

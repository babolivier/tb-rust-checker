use clap::Parser;

use crate::config::load_config_from_file;

mod checksums;
mod config;
mod matrix;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_file: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let cfg = match load_config_from_file(args.config_file) {
        Ok(cfg) => cfg,
        Err(err) => panic!("error parsing config: {err:?}"),
    };

    match matrix::sync(cfg, "".into()).await {
        Ok(_) => {}
        Err(err) => panic!("{err:?}"),
    }
}

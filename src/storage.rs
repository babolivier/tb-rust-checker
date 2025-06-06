use std::env;
use std::io::Error;
use std::path::PathBuf;

use tokio::fs;

use crate::config::Config;

const SYNC_TOKEN_FILE_NAME: &str = "matrix_sync_token.txt";

/// Build a path to the given store file using the configuration.
///
/// If the configuration does not specify a directory, the current working
/// directory is used instead.
fn get_path_in_store(cfg: &Config, file: &str) -> Result<PathBuf, Error> {
    let mut path = match &cfg.store_location {
        Some(path) => PathBuf::from(path),
        None => env::current_dir()?,
    };

    path.push(file);

    Ok(path)
}

/// Update the Matrix sync token in the on-disk store, so we don't need to
/// perform a full sync after the next restart.
pub(crate) async fn store_sync_token(cfg: &Config, token: &String) -> Result<(), Error> {
    let path = get_path_in_store(cfg, SYNC_TOKEN_FILE_NAME)?;
    log::debug!("Storing token {} at path {}", token, path.to_string_lossy());
    fs::write(path, token).await?;
    Ok(())
}

/// Read the Matrix sync token to use for the first sync at startup.
pub(crate) async fn read_sync_token_from_store(cfg: &Config) -> Result<String, Error> {
    let path = get_path_in_store(cfg, SYNC_TOKEN_FILE_NAME)?;
    let token = fs::read_to_string(path.clone()).await?;
    log::debug!("Read token {} at path {}", token, path.to_string_lossy());
    Ok(token)
}

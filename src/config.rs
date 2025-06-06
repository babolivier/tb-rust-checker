use std::fs;

use anyhow::Error;
use serde::Deserialize;

/// The user-defined config for the Matrix bot.
///
/// See the configuration sample file for documentation of each field.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    pub store_location: Option<String>,
    pub push_message_substring: String,
    pub messages: MessagesConfig,
    pub matrix: MatrixConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MessagesConfig {
    pub deps_out_of_date: String,
    pub deps_up_to_date: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MatrixConfig {
    pub server_host: String,
    pub access_token: String,
    pub room_id: String,
}

/// Read and parse the configuration file at the given path.
pub(crate) fn load_config_from_file(path: String) -> Result<Config, Error> {
    let file_content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&file_content)?;
    Ok(config)
}

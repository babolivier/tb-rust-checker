use anyhow::Error;
use serde::Deserialize;
use sha2::{Digest, Sha512};

#[derive(Debug, Clone, Deserialize)]
struct CommCentralChecksums {
    mc_workspace_toml: String,
    mc_gkrust_toml: String,
    mc_hack_toml: String,
    mc_cargo_lock: String,
}

const CC_CHECKSUMS_URL: &'static str =
    "https://hg-edge.mozilla.org/comm-central/raw-file/tip/rust/checksums.json";

const MC_WORKSPACE_TOML_URL: &'static str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/Cargo.toml";
const MC_GKRUST_TOML_URL: &'static str = "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/toolkit/library/rust/shared/Cargo.toml";
const MC_HACK_TOML_URL: &'static str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/build/workspace-hack/Cargo.toml";
const MC_CARGO_LOCK_URL: &'static str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/Cargo.lock";

pub(crate) async fn compare_checksums() -> Result<bool, Error> {
    let checksums: CommCentralChecksums = reqwest::get(CC_CHECKSUMS_URL).await?.json().await?;

    let futs = vec![
        compare_checksum_for_file(&checksums.mc_workspace_toml, MC_WORKSPACE_TOML_URL),
        compare_checksum_for_file(&checksums.mc_gkrust_toml, MC_GKRUST_TOML_URL),
        compare_checksum_for_file(&checksums.mc_hack_toml, MC_HACK_TOML_URL),
        compare_checksum_for_file(&checksums.mc_cargo_lock, MC_CARGO_LOCK_URL),
    ];

    let checksums_match = match futures::future::join_all(futs)
        .await
        .into_iter()
        .filter_map(|result| match result {
            Ok(checksum_matches) if checksum_matches => Some(checksum_matches),
            Ok(_) => None,
            Err(err) => {
                eprintln!("error querying files: {err}");
                None
            }
        })
        .next()
    {
        Some(_) => true,
        None => false,
    };

    Ok(checksums_match)
}

async fn compare_checksum_for_file(expected_checksum: &str, url: &str) -> Result<bool, Error> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let checksum = Sha512::digest(bytes);
    Ok(expected_checksum == hex::encode(checksum))
}

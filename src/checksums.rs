use serde::Deserialize;
use sha2::{Digest, Sha512};

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
struct CommCentralChecksums {
    mc_workspace_toml: String,
    mc_gkrust_toml: String,
    mc_hack_toml: String,
    mc_cargo_lock: String,
}

/// The file on comm-central containing the checksums to compare.
const CC_CHECKSUMS_URL: &str =
    "https://hg-edge.mozilla.org/comm-central/raw-file/tip/rust/checksums.json";

/// The files on mozilla-central to compare the checksums of.
const MC_WORKSPACE_TOML_URL: &str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/Cargo.toml";
const MC_GKRUST_TOML_URL: &str = "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/toolkit/library/rust/shared/Cargo.toml";
const MC_HACK_TOML_URL: &str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/build/workspace-hack/Cargo.toml";
const MC_CARGO_LOCK_URL: &str =
    "https://hg-edge.mozilla.org/mozilla-central/raw-file/tip/Cargo.lock";

/// Download the comm-central file containing the SHA512 checksums to compare,
/// then check if they match the checksums of the relevant mozilla-central
/// files.
///
/// This function returns whether the checksum of all files match the checksums
/// stored in comm-central.
pub(crate) async fn verify_checksums_match() -> Result<bool, Error> {
    // Download the checksums file from comm-central.
    let checksums: CommCentralChecksums = reqwest::get(CC_CHECKSUMS_URL).await?.json().await?;

    // Download all the relevant files, then compare their checksums to the ones
    // we expect.
    let futs = vec![
        compare_checksum_for_file(MC_WORKSPACE_TOML_URL, &checksums.mc_workspace_toml),
        compare_checksum_for_file(MC_GKRUST_TOML_URL, &checksums.mc_gkrust_toml),
        compare_checksum_for_file(MC_HACK_TOML_URL, &checksums.mc_hack_toml),
        compare_checksum_for_file(MC_CARGO_LOCK_URL, &checksums.mc_cargo_lock),
    ];

    let mismatch = futures::future::join_all(futs)
        .await
        .into_iter()
        .filter_map(|result| match result {
            // Record any mismatch in the checksums.
            Ok(checksum_matches) if !checksum_matches => Some(checksum_matches),
            Ok(_) => None,
            Err(err) => {
                log::error!("Error querying files to compare: {err}");
                None
            }
        })
        .next()
        .is_some();

    Ok(!mismatch)
}

/// Download the file at the given URL, then compare its SHA512 checksum to the
/// one that is expected as per the comm-central checksums file.
async fn compare_checksum_for_file(url: &str, expected_checksum: &str) -> Result<bool, Error> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let checksum = Sha512::digest(bytes);
    let checksum = hex::encode(checksum);

    log::debug!(
        "Comparing checksums for {}: {} == {}",
        url,
        expected_checksum,
        checksum
    );

    Ok(expected_checksum == checksum)
}

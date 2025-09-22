/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde::Deserialize;
use sha2::{Digest, Sha512};

use crate::error::Error;

/// The content of the `checksums.json` file on comm-central.
#[derive(Debug, Clone, Deserialize)]
struct CommCentralChecksums {
    mc_workspace_toml: String,
    mc_gkrust_toml: String,
    mc_hack_toml: String,
    mc_cargo_lock: String,
}

/// The repository on hg.mozilla.org to fetch a given file from.
enum Repo {
    /// The mozilla-central repository.
    MozillaCentral,

    /// The comm-central repository.
    CommCentral,
}

/// The file on comm-central containing the checksums to compare.
const CC_CHECKSUMS_PATH: &str = "rust/checksums.json";

/// The files on mozilla-central to compare the checksums of.
const MC_WORKSPACE_TOML_PATH: &str = "Cargo.toml";
const MC_GKRUST_TOML_PATH: &str = "toolkit/library/rust/shared/Cargo.toml";
const MC_HACK_TOML_PATH: &str = "build/workspace-hack/Cargo.toml";
const MC_CARGO_LOCK_PATH: &str = "Cargo.lock";

/// The revisions to use when querying files on hg.mozilla.org. Both fields
/// default to "tip" if not provided.
#[derive(Default)]
pub struct ChangeSet {
    /// The revision for mozilla-central.
    pub mc_rev: Option<String>,

    /// The revision for comm-central.
    pub cc_rev: Option<String>,
}

/// Download the comm-central file containing the SHA512 checksums to compare,
/// then check if they match the checksums of the relevant mozilla-central
/// files.
///
/// This function returns whether the checksum of all files match the checksums
/// stored in comm-central.
pub async fn verify_checksums_match(change_set: ChangeSet) -> Result<bool, Error> {
    // A helper macro that generates the URL to a raw file in the given repo on
    // the Mercurial web frontend.
    macro_rules! generate_url {
        ($repo:expr, $path:tt) => {{
            let (repo_name, rev) = match $repo {
                Repo::MozillaCentral => ("mozilla-central", &change_set.mc_rev),
                Repo::CommCentral => ("comm-central", &change_set.cc_rev),
            };

            format!(
                "https://hg-edge.mozilla.org/{}/raw-file/{}/{}",
                repo_name,
                rev.as_ref().unwrap_or(&"tip".to_string()),
                $path
            )
        }};
    }

    // Download the checksums file from comm-central. Downloading this
    // statically-served file should only result in 200 responses, so propagate
    // an error if we get an HTTP error.
    let checksums: CommCentralChecksums =
        reqwest::get(generate_url!(Repo::CommCentral, CC_CHECKSUMS_PATH))
            .await?
            .error_for_status()?
            .json()
            .await?;

    // Download all the relevant files, then compare their checksums to the ones
    // we expect.
    let futs = vec![
        compare_checksum_for_file(
            generate_url!(Repo::MozillaCentral, MC_WORKSPACE_TOML_PATH),
            &checksums.mc_workspace_toml,
        ),
        compare_checksum_for_file(
            generate_url!(Repo::MozillaCentral, MC_GKRUST_TOML_PATH),
            &checksums.mc_gkrust_toml,
        ),
        compare_checksum_for_file(
            generate_url!(Repo::MozillaCentral, MC_HACK_TOML_PATH),
            &checksums.mc_hack_toml,
        ),
        compare_checksum_for_file(
            generate_url!(Repo::MozillaCentral, MC_CARGO_LOCK_PATH),
            &checksums.mc_cargo_lock,
        ),
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
async fn compare_checksum_for_file(url: String, expected_checksum: &str) -> Result<bool, Error> {
    let bytes = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
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

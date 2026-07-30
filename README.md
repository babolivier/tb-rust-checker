# Rust Dependencies Checker for Thunderbird

As of [bug 1860654](https://bugzilla.mozilla.org/show_bug.cgi?id=1860654), which
adds the ability to build Thunderbird Desktop with Thunderbird-specific Rust
code, the Rust dependencies vendored in [the Thunderbird
repository](https://github.com/thunderbird/thunderbird-desktop) need to be kept
in sync with the ones vendored in [the Firefox
repository](https://github.com/mozilla-firefox/firefox/) (at least for
dependencies shared by both repositories).

In order to do this, two Thunderbird-specific
[`mach`](https://firefox-source-docs.mozilla.org/mach/index.html) commands were
introduced:

- `mach tb-rust check-upstream`, which checks whether the common dependencies
  between Thunderbird and Firefox are out of sync
- `mach tb-rust vendor`, which updates the Thunderbird manifest to ensure common
  dependencies are in sync, and revendor the Rust dependencies in Thunderbird.

The first command is performed automatically by the CI/CD infrastructure when a
new push to Firefox's `main` branch happens. The check's result is shared with
the Thunderbird sheriffs via a Matrix message; this same automation will then
also create a patch to sync the common dependencies.

## Problem

Due to the way this automation works, this Matrix message might be sent a long
time after the push has happened, which might be uncomfortable to sheriffs as
they might end up waiting quite a bit for it to arrive, only to be told there's
nothing that needs to be done. This is because the automation needs to clone
Thunderbird's and Firefox's repositories before running this command, which is
time-consuming, and because the CI/CD job itself might need to wait several
minutes for an available worker.

## Solution

This Matrix bot attempts to solve this issue by taking a different approach,
which is to replicate what `mach tb-rust check-upstream` does. This command
works by calculating SHA512 checksums of a few files in the Firefox repo and
comparing them to the values stored in a file in the Thunderbird one. The
Firefox files it checks are:

- `Cargo.toml`
- `Cargo.lock`
- `toolkit/library/rust/shared/Cargo.toml`
- `build/workspace-hack/Cargo.toml`

The bot listens to new messages in a specific Matrix room. When it sees a
message that indicates a new push to Firefox's main branch (also sent by the
CI/CD infrastructure), it downloads the contents of both the checksums file in
Thunderbird's repo, and the four Firefox files, from GitHub. It then compares
the checksums from the Firefox files with the ones stored in Thunderbird's repo,
and sends an appropriate notice to the Matrix room.

## How to use

Clone this repository and build the bot. Some system dependencies might be
necessary to build SSL support, such as `libssl-dev` and `pkg-config` on
Debian-based systems, as well as a Rust compiler.

```bash
sudo apt install libssl-dev pkg-config
git clone https://github.com/babolivier/tb-rust-checker.git
cd tb-rust-checker
cargo build
```

Then copy the [sample configuration file](/config.sample.toml), edit it
accordingly (using the documentation provided in the file itself), and use it to
run the bot.

```bash
cargo run -- -c config.toml
```

The `RUST_LOG` environment variable can be used to control logging. See the
documentation for the
[env_logger](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
crate.

## Command-line tool

This workspace also includes a command-line tool to manually run the file
verification logic. It can be run using Cargo:

```bash
cargo run --bin checker_cli
```

This tool can be used to compare files at given revisions of Firefox and
Thunderbird:

```
Usage: checker_cli [OPTIONS]

Options:
  -f, --firefox-rev <FIREFOX_REV>
          The Firefox revision to use. Defaults to "refs/heads/main"
  -t, --thunderbird-rev <THUNDERBIRD_REV>
          The Thunderbird revision to use. Defaults to "refs/heads/main"
  -h, --help
          Print help
```

For example, the following command compares the manifests between the commit
`AAA` on Firefox and `BBB` on Thunderbird:

```bash
cargo run --bin checker_cli -- -f AAA -t BBB
```

As mentioned in the help text above, each argument defaults to the latest
revision in the relevant repository's main branch. For example, the following
command compares the manifests between the latest commit of Thunderbird and the
commit `AAA` on Firefox:

```bash
cargo run --bin checker_cli -- -f AAA
```

The `RUST_LOG` environment variable can be used to control logging, in the same
way as with the main bot binary.

## Misc

### Does the bot also provide patches for Thunderbird?

No. Building the patch to bring the vendored Thunderbird Rust dependencies
up-to-date with Firefox still requires cloning both Thunderbird's and Firefox's
repositories, so it's best to leave it to the automation.

In theory, this bot _could_ make this step a bit quicker by using persistent
clones of both repositories, that it would simply update on every push to run
the `mach` command and generate a patch, but that would mean integrating this
bot with Mercurial and Phabricator, which I'm not interested in doing at the
moment.

### Why does the bot need on-disk storage?

The bot reads new messages sent to the Matrix room by polling the
[`/sync`](https://spec.matrix.org/v1.14/client-server-api/#get_matrixclientv3sync)
API in an almost-never-ending loop. In order for these syncs to be incremental,
a token can be provided when performing the request, which represents the point
at which the previous sync stopped.

If this token is not persisted on disk, then the first sync after a restart will
contain every message the bot has accessed to, rather than every message that
was sent after it last sync'd.

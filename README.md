# Rust Dependencies Checker for Thunderbird

As of [bug 1860654](https://bugzilla.mozilla.org/show_bug.cgi?id=1860654), which
adds the ability to build Thunderbird Desktop with Thunderbird-specific Rust
code, the Rust dependencies vendored in
[comm-central](https://hg.mozilla.org/comm-central) need to be kept in sync with
the ones vendored in [mozilla-central](https://hg.mozilla.org/mozilla-central)
(at least for dependencies shared by both repositories).

In order to do this, two Thunderbird-specific
[`mach`](https://firefox-source-docs.mozilla.org/mach/index.html) commands were
introduced:

- `mach tb-rust check-upstream`, which checks whether the common dependencies
  between comm-central and mozilla-central are out of sync
- `mach tb-rust vendor`, which updates the comm-central manifest to ensure
  common dependencies are in sync, and revendor the Rust dependencies in
  comm-central.

The first command is performed automatically by the CI/CD infrastructure when a
new push to mozilla-central happens. The check's result is shared with the
Thunderbird sheriffs via a Matrix message; this same automation will then also
create a patch to sync the common dependencies.

## Problem

Due to the way this automation works, this Matrix message might be sent a long
time after the push has happened, which might be uncomfortable to sheriffs as
they might end up waiting quite a bit for it to arrive, only to be told there's
nothing that needs to be done. This is because the automation needs to clone
comm-central and mozilla-central before running this command, which is
time-consuming, and because the CI/CD job itself might need to wait several
minutes for an available worker.

## Solution

This Matrix bot attempts to solve this issue by taking a different approach,
which is to replicate what `mach tb-rust check-upstream` does. This command
works by calculating SHA512 checksums of a few files in mozilla-central and
comparing them to the values stored in a file in comm-central. The
mozilla-central files it checks are:

- `Cargo.toml`
- `Cargo.lock`
- `toolkit/library/rust/shared/Cargo.toml`
- `build/workspace-hack/Cargo.toml`

The bot listens to new messages in a specific Matrix room. When it sees a
message that indicates a new push to mozilla-central (also sent by the CI/CD
infrastructure), it downloads the contents of both the checksums file in
comm-central, and the four mozilla-central files, using the web Mercurial
interface at <https://hg.mozilla.org/>. It then compares the checksums from the
mozilla-central files with the ones stored in comm-central, and sends an
appropriate notice to the Matrix room.

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

## Misc

### Does the bot also provide patches for comm-central?

No. Building the patch to bring the vendored comm-central Rust dependencies
up-to-date with mozilla-central still requires cloning both comm-central and
mozilla-central, so it's best to leave it to the automation.

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

### Why doesn't the bot use the [Matrix Rust SDK](https://github.com/matrix-org/matrix-rust-sdk/)?

I first attempted to build this tool using the Matrix Rust SDK, but encountered
a few issues with it. There are a few limitations in the SDK itself that make it
awkward to work with (e.g. converting a string to a `RoomId` isn't
straightforward when the string isn't a string literal, it's not possible to
access the latest sync token using the recommended way of performing syncs and
so a restart always causes a full sync and forces the bot to react to process
messages it's already seen, etc.); and the SDK makes projects more
resource-intensive to build than I think they need to be (converting this bot to
it and compiling it would take over 1GB of memory, even with incremental
compilation).

I'm open to integrating the Matrix Rust SDK in this project when these issues
are resolved; in the meantime using plain old `serde` and `reqwest` feels nicer.

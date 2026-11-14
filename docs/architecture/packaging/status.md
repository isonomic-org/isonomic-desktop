# Packaging: status

Newest first.

## 2026-11-14 (later) · Fetch task and bundle lookup ported; local only

**Built** (`packaging` crate and `cargo xtask fetch-sidecars`)

- The pinned manifest is carried over as `sidecars.json`, with every URL and hash unchanged. Entries for services outside Milestone 0 are marked `deferred`: pinned, validated, not fetched.
- Every manifest rule is ported and tested: versions and licenses required, both Apple slices required, no universal target declared, https only, hashes well-formed, a missing hash fails closed unless the entry is a draft or the run is recording.
- The installer: download, verify the SHA-256 against the pin, unpack, move into place, then check the Mach-O architecture against the target. An install already in place is architecture-checked again. Nothing is unpacked before its hash matches.
- `keepInBin`: a directory install keeps only the listed programs. PostgreSQL keeps `postgres` and `initdb`, the two the engine runs.
- Modes: default, `--check`, `--record`, `--target=`.
- `Bundle`: finds bundled binaries at run time. An explicit directory wins; inside an app bundle it is `Contents/Resources/sidecars`; otherwise `sidecars` beside the engine. The engine never looks on the user's `PATH`.
- Pure Rust: no Node, no shelling out to `tar`.

**Verified**

- Hermetic tests with an archive built in memory.
- A real run from Linux: both Apple builds of PostgreSQL 18.4.0 downloaded, matched their pinned hashes and passed the architecture check (x86_64 and arm64). The Intel hash also matches the vendor's published `.sha256` file.
- A Linux build was added and pinned from the vendor's published hash, so the engine could be run with the PostgreSQL it fetched itself. It ran end to end: first-time setup, healthy, force-kill and restart, ordered stop.

**Measured**

- PostgreSQL tree after trimming `bin`: 36 MB (Intel), 37 MB (Apple Silicon). Both ship in a universal app, so about 73 MB before any further trimming. The tree still carries headers and other files the engine never reads.

**Deviations**

- Only the directory install shape is implemented. The single-file shape (with the fuse into a universal binary) and build-from-source validate in the manifest and are refused by the installer until a service that needs them is bundled.
- New manifest fields: `deferred` and `keepInBin`.
- The downloader uses the system trust store and proxy settings, so it also works behind a corporate proxy.

**Deferred**

- Trimming the tree further (headers, unused shared files): measure in the Milestone 0 report.
- The universal `.app` and ad-hoc signing.

## 2026-11-14 · Not started

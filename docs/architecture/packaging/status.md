# Packaging: status

Newest first.

## 2026-11-14 (night) · The universal app (PR #4)

**Built** (`cargo xtask bundle` in this repo)

- `dist/Isonomic.app`, bundle id `com.isonomic.desktop`. The window and the engine are each built in release for `x86_64-apple-darwin` and `aarch64-apple-darwin` and fused with `lipo`. Both PostgreSQL trees ship under `Contents/Resources/sidecars/`; the engine picks the tree for its own slice, so the app needs no launcher logic.
- The engine repo must be cloned beside this one. The task runs its `cargo xtask fetch-sidecars`, which verifies every hash and architecture, cache hits included.
- From each PostgreSQL tree the app ships `bin` (`postgres`, `initdb`), `lib`, `share` and the two licence files. Headers and the upstream readme stay out.
- Signing is ad hoc: every Mach-O file inside, then the bundle. `codesign --verify --strict` must pass or the task fails.
- `--host-only` builds one architecture for quick local runs.

**Measured** (on the Intel test Mac)

| Part | Size |
| --- | --- |
| Window, both architectures | 42.4 MB |
| Engine, both architectures | 7.8 MB |
| PostgreSQL tree, Intel | 27.2 MB |
| PostgreSQL tree, Apple Silicon | 28.3 MB |
| **The app** | **106.0 MB** (one architecture only: 52.8 MB) |

- Leaving out the headers took each PostgreSQL tree from about 36 MB to 27 to 28 MB.
- First universal build from clean: about 75 minutes on the Intel Mac while other sessions were compiling (window: about 40 minutes for Intel, 26 for Apple Silicon; engine: 7 and 3). A rebuild with nothing changed takes seconds.
- `codesign --verify --strict` passes. `lipo` confirms both programs hold x86_64 and arm64.
- Launched from `dist/` the app runs the window, the engine and PostgreSQL from inside the bundle, and its Capsule goes to the default location under Application Support.

**Found on the Mac and fixed**

- Signing everything in one call failed: `codesign` refuses an app's main program while nested code is unsigned. Nested programs are now signed first and the bundle last.

**Deviations**

- The bundling task lives in this repo's xtask, not the engine's (the engine-supervisor spec lists `dist` there). The app's main program is the window, and the report that measures the app is here too.
- `LSMinimumSystemVersion` is 11.0, the first macOS that runs on Apple Silicon. The real minimum is open decision 1.
- No app icon: there is no approved design for one.
- `Cargo.lock` on `main` lacked the menu bar's dependencies because CI builds without `--locked`; this PR regenerates it.

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

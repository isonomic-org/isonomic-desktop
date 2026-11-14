# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone. Detail lives in the status files under `docs/architecture/`.

## 2026-11-14 (third entry) · Engine, SDK and window skeleton all exist; two repos still missing

**Done**

| Repo | Built | Tests | On GitHub |
| --- | --- | --- | --- |
| `isonomic-capsule` | `capsule-core`, `registry`, `supervisor`, `broker`, `packaging`, `xtask fetch-sidecars`, the engine binary | 74, 12 green runs in a row | No: the repo does not exist |
| `isonomic-sdk` | `proto`, `client`, `testkit` | 14, 12 green runs in a row | No: the repo does not exist |
| `isonomic-desktop` | `ui`, `engine-link`, `shell`, the window binary | 7, 6 green runs in a row | Yes: pull request #2 |

All three pass `cargo fmt` and `cargo clippy -D warnings` on Linux x86_64 with Rust 1.97.

**Proven end to end on Linux**

- The engine fetched its own pinned PostgreSQL 18.4, verified it, and ran it: first-time setup, healthy, force-kill and automatic restart, ordered stop with nothing left behind.
- Both Apple builds of PostgreSQL were downloaded and matched their pins and their architectures.
- The real shell, in a headless window, shows the rail and follows the database through healthy, restarting and healthy again.

**Not proven**

- Anything on macOS. Nothing has been compiled or run on a Mac.
- The window and the engine running together.

**Where the code is**

- The window is in pull request #2. It cannot build from GitHub until `isonomic-sdk` exists, because it uses a sibling checkout of the SDK.
- The engine and SDK are in the build sandbox and in the archive handed to Omar (`isonomic-workspaces.tar.gz`, all three workspaces).

**Blocked on Omar**

1. Create `isonomic-capsule` and `isonomic-sdk` and add CI: the GitHub connector is refused both (403). The bootstrap script does all of it.
2. Approve or change the message shapes (`docs/architecture/sdk/status.md`).
3. Decide the probe cadence (`docs/architecture/engine-supervisor/status.md`).

**Next, in order**

1. Once the repos exist: push the engine and SDK as pull requests, switch the sibling checkouts to pinned git dependencies, get CI green on Linux and macOS.
2. First real run on the Intel test machine, before any UI investment.
3. Menu bar icon spike, hide on close, Quit that stops the engine.
4. `cargo xtask report` and the universal unsigned app.

**Leftover to tidy**

- The empty branch `chore/ci-file-size` can be deleted. It was for a file-size check that could not be pushed because it is a workflow file.

## 2026-11-14 · Milestone 0, Step 0 (push, build, see)

| Check | Result | Evidence |
| --- | --- | --- |
| Repos exist | **Failed** | `isonomic-capsule`, `isonomic-sdk` and the org-level `.github` repo return 404. |
| Push | Works in this repo for ordinary files | Creating a repo and writing under `.github/workflows/` are refused. |
| Engine builds off-Mac | Confirmed | See the entry above. |
| Window builds off-Mac | Confirmed on Linux | GPUI Kit 0.9.1 compiles there. macOS itself is unverified. |
| Window builds on macOS CI | **Not possible yet** | There is no workflow. |
| File size | Rule adopted | Every source and doc file stays under 20 KB. |
| CI files | **Cannot be written** | They have to be added by Omar or by the bootstrap script. |

**Notes for whoever resumes**

- The build sandbox has no git access to this org. Reads and writes go through the GitHub connector, one file at a time, so the sandbox copy is the working tree and the connector push is the publish step.
- CI results are readable through the connector (check runs on a pull request).
- GPUI Kit needs these system packages to compile on Linux: `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libasound2-dev libvulkan-dev libx11-xcb-dev libfontconfig-dev libssl-dev`.

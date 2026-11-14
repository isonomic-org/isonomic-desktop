# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone.

## 2026-11-14 (later) · Engine runs end to end locally; nothing can be pushed yet

**Done**

- The spec is committed under `docs/architecture/`, one folder per area with `spec.md` and `status.md`. Read the status files for detail; this entry is the summary.
- Engine workspace (`isonomic-capsule`): `capsule-core`, `registry`, `supervisor`, `broker` and the `isonomic-capsule` binary. 58 tests, 60 consecutive green runs, fmt and clippy clean, on Linux x86_64 with Rust 1.97.
- SDK workspace (`isonomic-sdk`): `proto`, the v0 wire types, with the JSON of every message frozen in tests.
- The engine half of the Milestone 0 gate passes with the real binary against a real PostgreSQL 16 on Linux, driven over the socket. See `docs/architecture/plan/status.md`.

**Where the code is**

- Not on GitHub. The GitHub connector cannot create a repo in the org (403) and cannot write workflow files (403). Both workspaces exist in the build sandbox and as an archive handed to Omar (`isonomic-engine-and-sdk.tar.gz`).
- Once the repos exist the engine goes up as stacked pull requests in dependency order: engine core, registry, supervisor, PostgreSQL, broker and binary. The SDK goes up as one. A `README.md`, `CLAUDE.md`, `.gitignore` and a `ci.yml` (Linux and macOS: fmt, clippy, test) are in the archive.

**Blocked on Omar**

1. Create `isonomic-capsule` and `isonomic-sdk`, and add CI. The bootstrap script does all of it. Without it, two empty private repos are enough to push code, and the `ci.yml` from the archive has to be added by hand.
2. Approve or change the message shapes in `docs/architecture/sdk/status.md`.
3. Decide the probe cadence question in `docs/architecture/engine-supervisor/status.md`.

**Next, in order**

1. Push the engine and SDK pull requests; get CI green on Linux and macOS.
2. SDK `client` (reconnect, subscriptions) and `testkit` (in-memory engine).
3. Port the fetch task and pinned manifest (`xtask fetch-sidecars`) and the bundle resolver (`packaging`).
4. Window: `ui` re-export, `engine-link`, shell with the rail and Home tab, menu bar spike. Confirm the first window build on the Intel test machine before any UI investment.
5. `cargo xtask report`, then the universal unsigned app.

**Leftover to tidy**

- The empty branch `chore/ci-file-size` in this repo can be deleted. It was created for a file-size check that could not be pushed because it is a workflow file.

## 2026-11-14 · Milestone 0, Step 0 (push, build, see)

| Check | Result | Evidence |
| --- | --- | --- |
| Repos exist | **Failed** | `isonomic-capsule`, `isonomic-sdk` and the org-level `.github` repo return 404. This repo held only `README.md` and `.gitignore`. |
| Push | Works in this repo for ordinary files | Branches, commits and a pull request are created through the GitHub connector. Creating a repo and writing under `.github/workflows/` are refused. |
| Engine builds off-Mac | Confirmed | See the entry above. The crates.io registry is reachable from the build sandbox. |
| Window builds on macOS CI | **Not possible yet** | There is no workflow and no GPUI hello-world to compile. |
| File size | Rule adopted | Every source and doc file stays under 20 KB. |
| CI files | **Cannot be written** | The exact files have to be added by Omar or by the bootstrap script. |

**Notes for whoever resumes**

- The build sandbox has no git access to this org. Reads and writes go through the GitHub connector, one file at a time, so the sandbox copy is the working tree and the connector push is the publish step.
- CI results are readable through the connector (check runs on a pull request), so a paste-back is only needed for what CI cannot show: the window itself and `cargo xtask report` on the Intel test machine.

# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone.

## 2026-11-14 (later) · Engine core built and verified locally; repos still missing

**Done**

- The spec is committed under `docs/architecture/`, one folder per area with `spec.md` and `status.md`.
- Engine workspace built in the build sandbox: `capsule-core`, `registry`, `supervisor` (with the allow-listed child environment, Unix-socket endpoints and PostgreSQL first-time setup). 51 tests, 30 consecutive green runs, fmt and clippy clean.
- The engine half of the gate was run against a real PostgreSQL 16 on Linux: first-time setup, socket-only listening, cold start, force-kill then automatic restart, clean ordered stop. Numbers are in `docs/architecture/engine-services/status.md`.

**Where the code is**

- Not on GitHub. The connector cannot create `isonomic-capsule` (403), so the workspace exists in the build sandbox and as an archive handed to Omar (`isonomic-capsule-engine.tar.gz`).
- Once the repo exists it goes up as four stacked pull requests: engine core, registry, supervisor, PostgreSQL. A `README.md`, `CLAUDE.md`, `.gitignore` and a self-contained `ci.yml` (Linux and macOS: fmt, clippy, test) are prepared for it.

**Blocked on Omar**

1. Create `isonomic-capsule` and `isonomic-sdk` (run the bootstrap script, or create two empty private repos).
2. Decide the probe cadence question in `docs/architecture/engine-supervisor/status.md`.

**Next, in order**

1. Push the engine pull requests and get CI green on Linux and macOS.
2. SDK `proto` types for `system`, then the broker serving them, then `client` and `testkit`.
3. Port the fetch task and pinned manifest (`xtask fetch-sidecars`) and the bundle resolver (`packaging`), then the `isonomic-capsule` binary.
4. Window: `ui` re-export, `engine-link`, shell with the rail and Home tab, menu bar spike. Confirm the first window build on the Intel test machine before any UI investment.

## 2026-11-14 · Milestone 0, Step 0 (push, build, see)

| Check | Result | Evidence |
| --- | --- | --- |
| Repos exist | **Failed** | `isonomic-capsule`, `isonomic-sdk` and the org-level `.github` repo return 404. This repo held only `README.md` and `.gitignore`. |
| Push | Works in this repo | Branches, commits and a pull request are created through the GitHub connector. Creating a repo is refused. |
| Engine builds off-Mac | Confirmed | See the entry above. The crates.io registry is reachable from the build sandbox. |
| Window builds on macOS CI | **Not possible yet** | There is no workflow and no GPUI hello-world to compile. |
| File size | Rule adopted | Every source and doc file stays under 20 KB. |
| CI files | Untested | Whether the connector may write workflow files is not known yet. |

**Notes for whoever resumes**

- The build sandbox has no git access to this org. Reads and writes go through the GitHub connector, one file at a time, so the sandbox copy is the working tree and the connector push is the publish step.
- CI results are readable through the connector (check runs on a pull request), so a paste-back is only needed for what CI cannot show: the window itself and `cargo xtask report` on the Intel test machine.

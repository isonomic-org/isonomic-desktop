# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone. Detail lives in the status files under `docs/architecture/`.

## 2026-11-14 (fourth entry) · All three repos on GitHub; first runs on the Intel Mac

**Where the code is**

| Repo | Pull requests | CI |
| --- | --- | --- |
| `isonomic-sdk` | #1 proto, #2 client and testkit (stacked on #1) | Green on Linux and macOS |
| `isonomic-capsule` | #1 engine core, #2 broker, packaging and engine program (stacked on #1) | Stops at the SDK checkout: needs the deploy key |
| `isonomic-desktop` | #1 docs (this file), #2 window skeleton | Stops at the SDK checkout: needs the deploy key |

Pushed from Omar's Mac with the `gh` login for @QadmiriOmar, as Omar Qadmiri <omar@isonomic.com>. Working copies are in `~/Code/isonomic/repos`.

**Proven on the 2020 Intel MacBook Pro (macOS 15.7, Rust 1.98)**

- Engine: 74 tests pass. Both Apple builds of PostgreSQL 18.4 fetched and verified. The engine ran end to end twice: first-time setup, healthy, force-kill and restart in 6.1 s, clean shutdown, nothing left running. Cold start 5.0 s (probe cadence).
- SDK: 14 tests pass.
- Window: 7 tests pass, including the two headless window tests. The window binary builds with only the Command Line Tools (first build about 40 minutes), launches, starts the engine itself, and the engine runs PostgreSQL. Window process memory: 38 MB.

**Found on the Mac and fixed**

- macOS holds the first run of a brand-new script for 9 to 15 s on this machine. A ported test with a fixed 300 ms wait failed there; it now waits for the log line.
- The log pump stopped silently on a read error; it now writes the error to the log.

**Not proven**

- That the window draws correctly on screen: no person has looked at it yet.

**Deviations**

- No org-level shared workflow: each repo has its own `ci.yml`. The SDK is a sibling checkout pinned by the commit in `SDK_REF`, not a git dependency, so local builds and CI use the same layout. CI pins Rust 1.97.
- `gh` on the Mac is version 1.14 (2021) and holds one account, so it is now signed in as @QadmiriOmar only.

**Blocked on Omar**

1. Approve adding a read-only deploy key to `isonomic-sdk`, stored as the secret `SDK_DEPLOY_KEY` in `isonomic-capsule` and `isonomic-desktop`. Without it their CI cannot check out the private SDK.
2. Look at the window and say whether it shows the rail and a healthy database.
3. Probe cadence and message shapes, as before.

**Next, in order**

1. Deploy key, then CI green in all three repos.
2. Menu bar icon spike, hide on close, Quit that stops the engine.
3. `cargo xtask report`, then the universal unsigned app.

## 2026-11-14 (third entry) · Engine, SDK and window skeleton all exist; two repos still missing

| Repo | Built | Tests |
| --- | --- | --- |
| `isonomic-capsule` | `capsule-core`, `registry`, `supervisor`, `broker`, `packaging`, `xtask fetch-sidecars`, the engine binary | 74 |
| `isonomic-sdk` | `proto`, `client`, `testkit` | 14 |
| `isonomic-desktop` | `ui`, `engine-link`, `shell`, the window binary | 7 |

All three passed fmt, clippy and their tests on Linux x86_64 with Rust 1.97. The engine fetched its own pinned PostgreSQL and ran it end to end on Linux; the shell was tested in a headless window.

## 2026-11-14 · Milestone 0, Step 0 (push, build, see)

| Check | Result |
| --- | --- |
| Repos exist | Failed at first: `isonomic-capsule` and `isonomic-sdk` did not exist. Created later from the Mac |
| Push | The GitHub connector can push ordinary files to existing repos, but cannot create repos or write under `.github/workflows/` (403) |
| Engine builds off-Mac | Confirmed on Linux |
| Window builds off-Mac | Confirmed on Linux: GPUI Kit 0.9.1 compiles there |
| File size | Every source and doc file stays under 20 KB; CI checks it |

**Notes for whoever resumes**

- Pushing to new repos and workflow files goes through `git` on Omar's Mac with his `gh` login. The connector can still push docs and open pull requests, and its tokens cannot read check runs (403); use `gh api` on the Mac for CI status.
- GPUI Kit needs these system packages to compile on Linux: `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libasound2-dev libvulkan-dev libx11-xcb-dev libfontconfig-dev libssl-dev`.
- The empty branch `chore/ci-file-size` can be deleted.

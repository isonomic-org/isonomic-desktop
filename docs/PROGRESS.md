# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone. Detail lives in the status files under `docs/architecture/`.

## 2026-11-14 (fifth entry) · Milestone 0 gate: all four items pass on the Intel Mac

**Where the code is**

| Repo | Merged on `main` | Open |
| --- | --- | --- |
| `isonomic-sdk` | #1 proto, #2 client and testkit | none |
| `isonomic-capsule` | #1 engine core, #2 broker, packaging and engine program, #3 probe ramp | none |
| `isonomic-desktop` | #1 docs, #2 window skeleton, #3 menu bar, close to hide, Quit | #4 report and universal app: CI green, waiting for Omar's go-ahead |

CI is green on Linux and macOS in all three. The SDK is checked out by a read-only deploy key (`SDK_DEPLOY_KEY`). Working copies on the Mac: `~/Code/isonomic/repos`.

**Gate on the 2020 Intel MacBook Pro**

| Item | Result |
| --- | --- |
| 1. Window with the rail, database healthy | Pass, by eye in the dev bundle (evening entry in `plan/status.md`); engine side passes in the report |
| 2. Kill the database: it restarts, the window shows it | Pass, by eye; engine side passes (back in 1.4 to 3.0 s) in the report |
| 3. Quit stops everything in order | Pass, by eye; engine side passes (stopped in 0.1 to 0.2 s, nothing left) in the report |
| 4. The report prints cold start and bundle size | Pass: `cargo xtask report`; cold start 75 ms to 1.2 s depending on load; universal `Isonomic.app` 106.0 MB |

The pasted report block and all timed runs are in `plan/status.md`. The universal app, launched from `dist/`, shows the rail and "All services healthy".

**Not proven**

- The status item's colors as drawn and its own Open and Quit clicks: a person has to look once.
- Apple Silicon: the app carries an arm64 slice, but nothing has run on an Apple Silicon Mac.

**Deviations since the fourth entry**

- Bundling is `cargo xtask bundle` in this repo, not `dist` in the engine repo.
- The app ships PostgreSQL's `bin`, `lib`, `share` and licences, not its headers.
- `LSMinimumSystemVersion` 11.0 until open decision 1 is made. No app icon (no approved design).
- The window repo's CI pins Rust 1.98 (GPUI's macOS backend needs it); the engine and SDK pin 1.97.

**Open decisions for Omar**

1. Minimum macOS, and whether Intel Macs get Cells.
2. Keep running in the background as the default on Quit (today Quit always stops the engine).
3. The v0 message shapes in `sdk/spec.md`, which the code follows but nobody has approved.

**Next, in order** (Omar's queue, 14 Nov)

1. `isonomic-org/.github` with a shared `rust-ci.yml` (inputs: apt packages, extra steps, artifact upload path, Rust toolchain, a macOS job switch; builds with `--locked`). Move the engine and this repo onto it, then tell the Views and UI sessions it is live.
2. `Cargo.lock` committed and current in the engine and this repo (lockfiles are exempt from the 20 KB rule).
3. The `ui` crate moves from GPUI Kit 0.9.1 to the `isonomic-ui` crate on `main`, pinned the same way as the SDK.
4. Milestone 1: the remaining services, profiles, the RAM gate, logs, start at login.
5. The model-gateway core: router, local provider through `isonomic-inference`, the real RAM gate, per-app metering and the global toggle, serving the SDK's models interface.

Then the agents adapter, `isonomic-capsule/crates/agents` ("Agent host wiring", Part A): adapters over the harness host's traits only, with no loop, registry or policy code in the engine.

**Handed over:** `isonomic-sdk` belongs to the SDK session (proto, client, testkit). This session implements the engine and window side of what it defines; it files requests in `isonomic-capsule/docs/requests.md`. The Harness and Chat sessions file theirs in `docs/requests.md` too; Chat's crate is linked at a pinned git revision when it asks.

Every Mac check and timing is on Omar's Intel Mac; Apple Silicon is covered by CI only. Nothing merges without Omar's go-ahead in chat.

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

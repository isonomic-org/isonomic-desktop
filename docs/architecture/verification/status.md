# Verification: status

Newest first.

## 2026-11-14 (night) · `cargo xtask report` (PR #4)

**Built**

- `cargo xtask report [--host-only] [--no-build]`. It builds the app (or reuses `dist/`), starts the app's engine against a scratch Capsule, and drives it through the SDK client exactly as the window does.
- Item 1: waits for healthy (up to 180 s: first start includes first-time database setup and macOS's one-off check of new programs).
- Item 2: `kill -9` on the postmaster from its pid file, then waits for PostgreSQL to be healthy with a higher restart count (up to 90 s). Passes only if the new postmaster has a new pid and none of the old one's workers are left.
- Item 3: `system.shutdown`, which is what Quit sends. Passes only if the engine exits 0, PostgreSQL's log shows a completed shutdown and its pid file is gone, the engine's log ends with `stopped` after the request, the socket is removed, and no engine or PostgreSQL process is left. Exited-but-unreaped processes count as gone.
- Item 4: cold start (the engine's own figure, plus time from launch to healthy) and the app's size, part by part.
- The block also carries the machine, toolchain, version and git revision of each repo, both `SDK_REF`s, the PostgreSQL version and the engine's lines saying where each binary resolved. On failure it adds the last 25 lines of the engine and PostgreSQL logs and keeps the scratch Capsule. It is saved to `dist/report.txt`.
- On Linux it runs the engine repo's release build with its fetched `sidecars/`; item 4 is n/a there.

**Verified**

- Linux, as a non-root user: items 1 to 3 pass (PostgreSQL back in 1.2 s, 0 of 8 old workers left; stopped in 0.1 s, 0 of 10 processes left). With the engine missing, item 1 fails, 2 and 3 are skipped and the scratch Capsule is kept.
- Intel test Mac: the universal app passes all four items. Runs:

| Run (Intel Mac) | Cold start (engine) | Healthy after launch | Recovery after kill | Stop |
| --- | --- | --- | --- | --- |
| Host-only app, first run of new binaries | 626 ms | 11.8 s | 1.9 s | 0.1 s |
| Universal app, first run of new binaries | 313 ms | 7.1 s | 1.5 s | 0.1 s |
| Universal app, quiet machine | 75 ms | 1.8 s | 1.4 s | 0.1 s |
| Universal app, another session running a release build | 1186 ms | 54.1 s | 3.0 s | 0.2 s |

"Healthy after launch" includes first-time database setup, which every run does because each report uses a fresh scratch Capsule. The first run of new binaries also waits for macOS's one-off check of programs it has not seen. Every timing here is on the Intel Mac; Apple Silicon is covered by CI only.

The pasted block is in `plan/status.md`.

**Not covered**

- The window's half of the gate (the rail, the light through kill and recovery, Quit from the menu) stays a check by eye. The report says so in its block.

## 2026-11-14

- **Engine:** the hermetic suite exists for what is built (51 tests) and runs in the build sandbox. No CI runs it yet.
- **Real-service check:** `pg_smoke`, a manual example in the supervisor crate, runs the engine side of the Milestone 0 gate against a real PostgreSQL. It is the seed of `cargo xtask report`.
- **Window:** nothing yet.
- **Report command:** not started.

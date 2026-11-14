# Milestones and gate: status

Newest first.

## 2026-11-14 (evening) · Gate items 1 to 3 pass on the Intel test Mac

Run on the 2020 Intel MacBook Pro (macOS 15.7), with the window in a dev app bundle (`Isonomic Dev.app`, ad-hoc signed) holding the window, the engine and PostgreSQL 18.4.

| Gate item | Result |
| --- | --- |
| 1. Launching shows the window with the rail and the database healthy | **Pass.** The window started the engine itself; it showed the rail and "All services healthy" with postgres green |
| 2. Force-killing the database: it restarts and the window shows degraded then healthy, untouched | **Pass.** After `kill -9` the window showed "Services starting" with postgres Restarting (amber), then "All services healthy" with "1 restart", without being touched. The engine reports a crash as starting, not degraded; degraded is the circuit breaker |
| 3. Quit stops everything in order, nothing left behind | **Pass.** Quit from the app menu: window, engine and PostgreSQL all gone within 1 s; PostgreSQL logged a clean shutdown and removed its lock file; the engine socket was removed |
| 4. The report prints cold-start time and bundle size | **Not yet.** Cold start is logged (about 0.2 s since the probe ramp). No report command and no universal app yet |

Also verified: closing the window hides the app while the engine and database keep running; launching the app again shows the window; the menu bar status item is on screen.

Not verified: the status light's colors as drawn, and clicking the menu bar item's own Open and Quit. No background tool can see or click a status item; a person has to look once.

Still to build: `cargo xtask report` and the universal unsigned app.

## 2026-11-14 (later still) · Both halves exist; nothing has run on a Mac

Engine side of items 1 to 3 proven on Linux; window side proven in headless window tests. Cold start was 5.0 s, all of it probe cadence.

## 2026-11-14 · Milestone 0 in progress; gate not attempted

# Milestones and gate: status

Newest first.

## 2026-11-14 (later still) · Both halves exist; nothing has run on a Mac

| Gate item | Engine side | Window side |
| --- | --- | --- |
| 1. Window shows the rail and a healthy database | Proven on Linux with the PostgreSQL the engine fetched itself | Proven in a headless window test |
| 2. Force-kill recovers and the window shows it | Proven: `starting` at once, `healthy` with one restart 6.0 s later | Proven in a headless window test: restarting, then healthy, untouched |
| 3. Quit stops everything in order, nothing left behind | Proven: exit code 0, no process left, socket removed | Not started: no Quit in the window yet |
| 4. Report prints cold-start time and bundle size | Cold start is logged and reported: 5.0 s, all of it probe cadence. PostgreSQL trees measured: about 73 MB for both architectures | No report command yet |

The two halves have not been run together: the headless test feeds the window the status the engine link would publish. The first real run, with the real window talking to the real engine, has to happen on the Intel test machine.

Still to build before the gate can be attempted: the menu bar icon, hide on close, Quit, the universal app, and `cargo xtask report`.

## 2026-11-14 · Milestone 0 in progress; gate not attempted

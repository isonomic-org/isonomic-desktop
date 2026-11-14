# Milestones and gate: status

Newest first.

## 2026-11-14 (later) · Engine half of the gate passes on Linux

| Gate item | Status |
| --- | --- |
| 1. Window shows the rail and a healthy database | Engine side ready: `system.subscribe` reports the database healthy. No window yet |
| 2. Force-kill recovers and the window shows it | Engine side proven with the real binary and a real PostgreSQL: `starting` event at once, `healthy` with one restart 6.0 s later. No window yet |
| 3. Quit stops everything in order, nothing left behind | Engine side proven: `system.shutdown` is acknowledged, exit code 0, no process left, socket removed. No window yet |
| 4. Report prints cold-start time and bundle size | The engine logs and reports cold start (5.0 s, all of it probe cadence). No report command, no bundle yet |

Not yet run anywhere on macOS. The gate itself is on Omar's Intel Mac and needs the window, the bundled PostgreSQL and the report command.

## 2026-11-14 · Milestone 0 in progress; gate not attempted

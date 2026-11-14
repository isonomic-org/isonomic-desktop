# Milestones and gate: status

Newest first.

## 2026-11-14 · Milestone 0 in progress; gate not attempted

| Gate item | Status |
| --- | --- |
| 1. Window shows the rail and a healthy database | Not started (no window, no broker) |
| 2. Force-kill recovers and the window shows it | Engine half proven against a real PostgreSQL on Linux: restart on its own in 6.0 s. Window half not started |
| 3. Quit stops everything in order, nothing left behind | Engine half proven on Linux. Window half not started |
| 4. Report prints cold-start time and bundle size | Not started. Cold start measured by hand: 5.0 s, all of it probe cadence |

Open question raised by the numbers: the probe cadence while starting (see `engine-supervisor/status.md`).

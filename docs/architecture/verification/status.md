# Verification: status

Newest first.

## 2026-11-14

- **Engine:** the hermetic suite exists for what is built (51 tests) and runs in the build sandbox. No CI runs it yet.
- **Real-service check:** `pg_smoke`, a manual example in the supervisor crate, runs the engine side of the Milestone 0 gate against a real PostgreSQL. It is the seed of `cargo xtask report`.
- **Window:** nothing yet.
- **Report command:** not started.

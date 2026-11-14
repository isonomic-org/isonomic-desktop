# Services: status

Newest first.

## 2026-11-14 · PostgreSQL supervised end to end; built locally, not yet on GitHub

**Built** (`supervisor::postgres`)

- First-time setup: `initdb` runs once, into a staging directory that is renamed into place only on success. A failed or interrupted setup leaves nothing that looks like a cluster. The bootstrap role is `isonomic`, encoding UTF-8, locale C.
- Exposure: a Unix socket in a user-only directory under the Capsule run directory. `listen_addresses` is empty, so there is no network listener at all. Local connections use trust auth; host connections are rejected.
- Readiness: the wire-protocol probe (startup handshake, then `SELECT 1`) over the socket.
- Unclean shutdown: detected from the leftover lock file. The start deadline is then 300 s, against 90 s after a clean stop.
- Stop: the fast-shutdown signal, then the usual grace and forced stop.

**Verified**

- Hermetic tests with a stand-in `initdb` and a fake server on a Unix socket.
- A manual run against a real PostgreSQL 16 on Linux, with misleading database variables set on the engine (`PGPORT`, `PGHOST`, `PGDATA`):
  - first-time setup created the cluster; the second run reused it
  - cold start 5.0 s (the probe cadence, see the supervisor status; the server itself is ready in about 0.1 s)
  - force-kill of the server: restarted on its own, healthy again after 6.0 s, restart counted in the registry
  - ordered stop: clean shutdown, lock file removed, no process left behind
- The command is `cargo run -p supervisor --example pg_smoke -- <postgres bin dir> <scratch dir>`.

**Deferred**

- Bundled binaries: the fetch task and the pinned manifest are not ported yet, so the engine has no PostgreSQL of its own to run.
- Profile-driven settings such as buffer sizes: Milestone 1.
- One database and one role per app: Milestone 4.
- An orphaned server from a crashed engine: the lock file is detected and recovery time is allowed, but a still-running orphan is not hunted down.
- A run on macOS.

**Not started**

- Every other service. Out of scope for Milestone 0.

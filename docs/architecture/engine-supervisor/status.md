# Engine core and supervisor: status

Newest first. Every deviation and deferral is recorded here.

## 2026-11-14 (later) · Startup probe ramp; Mac results (isonomic-capsule PRs #1 to #3)

**Changed, with Omar's approval** (PR #3)

- While a service starts, readiness probes begin 50 ms apart and double up to the configured interval (5 s), instead of waiting the full interval after the first miss. This applies to the first start and to restarts. The spec's health line is updated to match.

| Measured on the Intel test Mac, real PostgreSQL 18.4 | Before | After |
| --- | --- | --- |
| Cold start | 5.0 s | about 0.2 s |
| Force-kill to healthy | 6.1 s | 1.8 to 2.7 s |

Recovery is now mostly the 1 s restart backoff plus PostgreSQL's own crash recovery.

**Found on the Mac and fixed**

- macOS holds the first run of a brand-new script for 9 to 15 s on the test machine. A ported test that waited a fixed 300 ms failed there; it now waits for the log line itself, up to 90 s.
- The log pump stopped without a trace on a read error. It now writes the error into the service's log.

**Verified**

- 75 tests on Linux and on the Intel Mac; CI green on Linux and macOS for PRs #1 and #2.
- One unexplained failure in one Mac run after the ramp change; it did not reproduce in six further runs.

**Deferred**, unchanged from below.

## 2026-11-14 · Ported and extended

**Built**

- `capsule-core`: service states and tiers, the Capsule directory layout, engine config, an injectable clock, the hardware trait and profiles.
- `supervisor`: service lifecycle, crash monitor, hang probe, circuit breaker, log sink and startup stack, carried over with behaviour unchanged and their tests ported.
- New: every child's environment is built from an allow-list (`ChildEnv`). The environment is cleared at spawn and only declared variables are set. The baseline is a fixed `PATH`, the C locale and UTC. A test proves database, loader and home variables set on the engine never reach a child.
- New: a service can listen on a Unix socket (`Endpoint::Unix`). Templates gained `{socket}` and `{socket_dir}`. Every probe runs over either kind of endpoint.
- The Capsule layout keeps socket paths under the OS limit: sockets live in `<root>/run`, or in a short per-user directory under `/tmp` when the root is too deep. The run directory is user-only and one owned by another user is refused.

**Deviations**

- `service.rs` is split into `service.rs`, `spawn.rs` and `monitor.rs` to keep every file under 20 KB. No behaviour change.
- Hardware profiles live in `capsule-core::hardware`, not under `supervisor` as section 4.1 lists. The brief puts the hardware trait in `capsule-core` and the two belong together.
- `ServiceSupervisor::addr()` is now `endpoint()`, and `start()` returns an `Endpoint` instead of a socket address.
- Not ported: the service definitions for the cache, object storage and proxy, the model supervisor, the model downloader, the RAM gate and the routing module. They are out of scope for Milestone 0 or dropped by the design.

**Deferred**

- Pausing dependents when a service degrades: tier 3 does not exist yet.
- The probe trigger "immediately when a consumer reports an error": needs the broker to accept such reports.
- The RAM gate: lands with the model gateway.

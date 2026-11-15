# Engine core and supervisor: status

Newest first. Every deviation and deferral is recorded here.

## 2026-11-15 (later) · A second engine, quoted paths, left-behind services (isonomic-capsule PRs #21 to #23)

- A second engine for the same Capsule exits 0 instead of failing (#21), so the login agent doesn't restart a duplicate in a loop.
- Services left running by a killed engine are stopped before the next engine starts its own (#23). `ServiceConfig::pid_file` (opt-in; ported behaviour unchanged): every spawn notes the pid, a deliberate stop removes it, and start stops whatever it names, if alive and running a program of the same name, with the service's stop signal and grace. PostgreSQL and the cache have pid files in the run directory. This closes the "still-running orphan" deferral below.
- Measured on the Intel Mac through the login agent: `kill -9` on the engine to both services healthy, 5.1 s.


## 2026-11-15 · Profiles applied, the lazy tier and the RAM gate (isonomic-capsule PRs #17, #19, #20)

**Built**

- Profiles drive configuration (#17): the engine picks one at start (detected from total memory, or `ISONOMIC_PROFILE`; an unreadable override is logged and a fresh probe used) and renders it into each service's config: database buffers and cache memory per the table. The log line says which profile and why.
- The lazy tier (#19): `Stack::ensure_started` starts a tier-2 service on first use. Concurrent callers share one start; a failed start reaches every caller that waited on it, and the next call tries again. A tier-2 service that hasn't started is left out of `system.status`, so it doesn't hold the light off green.
- The RAM gate (#20), carried over from the earlier build with its tests: the profile ceiling first (4 GiB Low, 10 GiB Standard, none on High), then free memory plus 2 GiB headroom. The refusal has the SDK's `memory` error fields and points to a smaller model or a remote one. A tier-2 service registered with a footprint is checked before every start; a refusal starts nothing. A gated service with no gate is refused, not started unchecked.

**Decision: one gate.** The inference workspace declares the same seam and ships only a fixed-budget gate for tests, leaving the hardware-aware gate to the engine. So the engine's gate is the only real one; the model gateway adapts it to the inference trait when it takes that dependency.

**Deviations**

- `Stack::aggregate_state` keeps its ported behaviour (an idle tier-2 service counts as stopped). The broker filters idle lazy services out of the status instead.
- The stack tests count launches from the supervisor's log, not from the stand-in script: on the Intel Mac a freshly written script can miss a whole two-second start (the first-run hold noted below).

**Deferred**

- Model workers behind the gate, and a refusal on the wire (`insufficient_memory`): the model gateway, with the SDK's R2.
- The other profile rows (model idle unload, apps kept loaded) have values but no consumer yet.
- Pausing dependents and the consumer-reported probe trigger, unchanged from below.

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

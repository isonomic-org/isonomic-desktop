# Engine core and supervisor: status

Newest first. Every deviation and deferral is recorded here.

## 2026-11-14 · Ported and extended; built locally, not yet on GitHub

**State.** Built and verified in the build sandbox. It cannot be pushed until the `isonomic-capsule` repo exists: the GitHub connector is not allowed to create repos in the org.

**Built**

- `capsule-core`: service states and tiers, the Capsule directory layout, engine config, an injectable clock, the hardware trait and profiles.
- `supervisor`: service lifecycle, crash monitor, hang probe, circuit breaker, log sink and startup stack, carried over with behaviour unchanged and their tests ported.
- New: every child's environment is built from an allow-list (`ChildEnv`). The environment is cleared at spawn and only declared variables are set. The baseline is a fixed `PATH`, the C locale and UTC. A test proves database, loader and home variables set on the engine never reach a child.
- New: a service can listen on a Unix socket (`Endpoint::Unix`). Templates gained `{socket}` and `{socket_dir}`. Every probe runs over either kind of endpoint.
- The Capsule layout keeps socket paths under the OS limit: sockets live in `<root>/run`, or in a short per-user directory under `/tmp` when the root is too deep. The run directory is user-only and one owned by another user is refused.

**Verified**

- 51 tests across `capsule-core`, `registry` and `supervisor`, 30 consecutive runs green, on Linux x86_64 with Rust 1.97. `cargo fmt` and `cargo clippy -D warnings` clean.
- One flake found and fixed on the way: a stand-in service could be signalled before its handler was installed. The stand-in now reports when it is ready.

**Deviations**

- `service.rs` is split into `service.rs`, `spawn.rs` and `monitor.rs` to keep every file under 20 KB. No behaviour change.
- Hardware profiles live in `capsule-core::hardware`, not under `supervisor` as section 4.1 lists. The brief puts the hardware trait in `capsule-core` and the two belong together.
- `ServiceSupervisor::addr()` is now `endpoint()`, and `start()` returns an `Endpoint` instead of a socket address.
- Not ported: the service definitions for the cache, object storage and proxy, the model supervisor, the model downloader, the RAM gate and the routing module. They are out of scope for Milestone 0 or dropped by the design.

**Needs a decision from Omar**

- Probe cadence while starting. The ported code waits a full 5 seconds after the first missed probe, so a cold start measures 5.0 s although PostgreSQL is ready in about 0.1 s, and recovery after a force-kill measures 6.0 s. A ramp (50 ms doubling up to 5 s) would bring both down to well under a second plus backoff. This changes ported behaviour and the spec line "every 5 seconds while starting", so it is not done without approval.

**Deferred**

- Pausing dependents when a service degrades: tier 3 does not exist yet.
- The probe trigger "immediately when a consumer reports an error": needs the broker.
- The RAM gate: lands with the model gateway.
- A macOS run of the suite: no CI exists yet.

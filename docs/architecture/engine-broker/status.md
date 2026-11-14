# Broker: status

Newest first.

## 2026-11-14 (later) · System interface built; local only, not yet on GitHub

**Built**

- `broker`: a Unix socket server speaking the v0 wire format. Version negotiation, then `system.status`, `system.subscribe`, `system.unsubscribe`, `system.logs`, `system.profile` and `system.shutdown`.
- One writer task per connection owns the write half, so frames never interleave. A subscription starts listening before it takes its snapshot, so no change falls between the two. A subscriber that falls behind is resent every service's current status.
- Single instance: an advisory lock on `engine.lock` beside the socket. The kernel drops it when its holder dies, so a crashed engine never blocks the next one and a live one always does. The socket file is user-only.
- `StackSystem`: the adapter that presents the supervisor's stack, the registry and the hardware profile in wire types. The status light is derived from the same readings as the service list.
- `bin/isonomic-capsule`: binds the socket first, logs where each bundled binary resolved and whether it is executable, runs PostgreSQL first-time setup, starts tier 1, logs the cold-start time, serves until `system.shutdown`, SIGTERM or SIGINT, then stops everything in reverse order and removes the socket. A failed start does not end the engine: it keeps serving status.

**Verified**

- Hermetic tests over a real Unix socket with a fake engine, plus one that drives the real supervisor with a stand-in service: a force-kill is seen by a subscriber as starting, then healthy with the restart counted. 58 tests in the engine workspace, 60 consecutive green runs.
- The real binary against a real PostgreSQL 16 on Linux, driven over the socket by a small script: welcome, subscribe during startup, healthy; force-kill of the server gave a `starting` event at once and `healthy` with one restart 6.0 s later; `system.shutdown` was acknowledged, the engine exited 0, no process was left and the socket was gone. A second engine on the same Capsule was refused.

**Deviations**

- The first single-instance check probed the socket by connecting to it. A test showed that is unreliable (a descriptor briefly copied by a starting child keeps a dead socket answering), so it was replaced by the lock before anything shipped.
- The rule for where bundled binaries live (`bundle/` beside the engine, or `ISONOMIC_BUNDLE_DIR`) sits in the binary for now. It moves to the `packaging` crate when that is ported.

**Deferred**

- Sessions and capability tokens: the app model, Milestone 4. Any local process running as the user can call `system` today.
- Recording calls for the Activity tab.
- The previous protocol version: there is only one so far.

## 2026-11-14 · Not started

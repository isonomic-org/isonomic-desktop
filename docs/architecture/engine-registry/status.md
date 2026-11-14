# Registry: status

Newest first.

## 2026-11-14 · Observed state ported; built locally, not yet on GitHub

**Built**

- The service table carried over: allocated port, lifecycle state, restart count, last-healthy time, and the aggregate status rule.
- New: a `socket` column and `record_socket`, for services that listen on a Unix socket.
- New: the last-healthy stamp comes from an injected clock, so its test no longer reads the wall clock.
- Persist-then-publish is kept where it was: the supervisor writes the registry row before it sends on the state channel. The ported test that caught the original race is included.

**Verified**

- 4 tests, part of the 51 that pass in the engine workspace.

**Deferred**

- Desired state, grants, schedules, app manifests and the audit index. The registry holds observed state only for now. Desired state arrives with the broker, the rest with their milestones.
- A schema version and migrations: needed before the first release that carries real user state.

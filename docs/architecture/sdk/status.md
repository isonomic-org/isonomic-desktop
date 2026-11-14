# SDK: status

Newest first.

## 2026-11-14 · Not started

The `isonomic-sdk` repo does not exist yet. Next after the engine core: `proto` types for `system`, then the broker that serves them, then `client` and `testkit`.

One decision recorded ahead of the code: the rule for where the engine socket lives will move into `proto`, so the engine and the window compute the same path from one implementation. Today it lives in the engine's `capsule-core`.

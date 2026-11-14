# SDK: status

Newest first.

## 2026-11-14 (later) · `proto` built; local only, not yet on GitHub

**Built** (`isonomic-proto`)

- Framing: a 4-byte big-endian length, then that many bytes of UTF-8 JSON. Frames over 1 MiB are refused.
- Messages: `hello` and `request` from the client; `welcome`, `response` and `event` from the engine. A request carries a client-chosen `id`, a `method` such as `system.status`, and `params` when the method has any. A response carries `result` or `error`.
- Types for `system`: service status, system status with the cold-start time, hardware profile, and error codes.
- Version negotiation: the highest version both sides list.
- Async frame reading and writing over any byte stream, behind a `tokio` feature, shared by the engine and the future client.

**Verified**

- 5 tests. Two of them freeze the exact JSON of every message kind. If one of those strings has to change, the protocol version changes with it.

**Needs Omar's approval**

- The message shapes. The spec fixes the framing, the three message kinds, negotiation and the list of `system` calls, but not the JSON itself. The shapes above were chosen so the engine could be built and tested; they are cheap to change until the window depends on them. Examples:

```
{"type":"hello","versions":[0],"client":"isonomic-desktop"}
{"type":"welcome","version":0,"engine":"0.0.1"}
{"type":"request","id":2,"method":"system.logs","params":{"service":"postgres","limit":20}}
{"type":"response","id":4,"error":{"code":"unknown_service","message":"no service named cache"}}
{"type":"event","subscription":7,"event":"service_status","data":{"name":"postgres","tier":1,"state":"healthy","restart_count":2,"last_healthy_unix":1700000000}}
```

**Deferred**

- `client` (reconnect and subscriptions) and `testkit` (an in-memory engine): next.
- Moving the rule for where the engine socket lives from the engine's `capsule-core` into `proto`, so the engine and the window compute the same path from one implementation.
- How the engine repo depends on this one once both are on GitHub: a pinned git dependency on a private repo needs CI credentials. Today it is a sibling checkout.
- The `app` crate: needs Isonomic UI.

## 2026-11-14 · Not started

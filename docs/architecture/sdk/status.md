# SDK: status

Newest first.

## 2026-11-14 (later still) · `client` and `testkit` built; local only

**Built**

- `isonomic-client`: one multiplexed connection run by a background task. It connects, negotiates the version, subscribes to service status, and reconnects with backoff. Callers get a watch on the live view (connection plus latest system status, kept current from the subscription) and request methods that fail fast while the engine is unreachable.
- `isonomic-testkit`: an in-memory engine that speaks the real protocol over in-memory streams. Tests script it: change a service, take the engine offline and back, check whether shutdown was requested.
- `proto::locate`: the rule for where a Capsule and its engine socket live, moved here from the engine so both sides compute the same path. Its output is frozen in a test.
- `SystemStatus::apply` and `ServiceState::aggregate`, shared by the engine and the client.

**Verified**

- 14 tests, 12 consecutive green runs: requests and typed errors, the live view following changes without polling, reconnect and resubscribe after the engine goes away, a client started before the engine.

**Design notes**

- The background task needs a Tokio runtime; the request methods and the watch do not, so a view can await them on GPUI's executor.
- Reading a frame cannot be cancelled halfway without losing bytes, so a dedicated task reads whole messages and hands them over.

## 2026-11-14 (later) · `proto` built

- Framing: a 4-byte big-endian length, then that many bytes of UTF-8 JSON. Frames over 1 MiB are refused.
- Messages: `hello` and `request` from the client; `welcome`, `response` and `event` from the engine.
- The exact JSON of every message kind is frozen in tests. If one of those strings has to change, the protocol version changes with it.

**Needs Omar's approval**

- The message shapes. The spec fixes the framing, the three message kinds, negotiation and the list of `system` calls, but not the JSON itself. Examples:

```
{"type":"hello","versions":[0],"client":"isonomic-desktop"}
{"type":"welcome","version":0,"engine":"0.0.1"}
{"type":"request","id":2,"method":"system.logs","params":{"service":"postgres","limit":20}}
{"type":"response","id":4,"error":{"code":"unknown_service","message":"no service named cache"}}
{"type":"event","subscription":7,"event":"service_status","data":{"name":"postgres","tier":1,"state":"healthy","restart_count":2,"last_healthy_unix":1700000000}}
```

**Deferred**

- How the engine and window repos depend on this one once it is on GitHub: a pinned git dependency on a private repo needs CI credentials. Today both use a sibling checkout.
- The `app` crate: needs Isonomic UI.

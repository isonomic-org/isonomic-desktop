# Services

Spec section 4.5.

| Service | v1 choice | Notes |
| --- | --- | --- |
| SQL | PostgreSQL, bundled | One cluster, one database per app, one role per app. Unix socket only |
| Cache and queue | The service already pinned and packaged in the earlier build | Unix socket. Persistence rules set per use |
| Object storage | The service already pinned and packaged | Loopback with a random port and secret. Reconsider an in-engine store after Milestone 3 |
| Vector | Single-binary vector database | Lazy. Memory-mapped on Low |
| Local models | llama.cpp server now; speech and image workers later | Lazy, gated, idle unload |
| Reverse proxy | None | Apps are not web apps |

The bundle carries only what PostgreSQL needs to run (about five of its roughly 35 tools), which cuts the largest part of the download.

## PostgreSQL

- The engine runs first-time setup.
- It detects an unclean shutdown and allows recovery time before declaring ready.
- It stops with the fast-shutdown signal.
- It listens on a Unix socket inside the Capsule directory, not on a network port.
- Never reuse a database the user already has installed.

## Rules for every service

- Everything the engine runs ships inside the app, pinned and verified.
- Every service has a protocol-level readiness probe. No probe shells out to a client tool.
- Every service stops with its own signal and deadline.
- Heavy services start on first use.

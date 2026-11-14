# Architecture

Isonomic Desktop, Architecture v0.2 (Rust + GPUI, from scratch), split by area. Each folder holds:

- `spec.md`: what the area must do. Changes only when the design changes.
- `status.md`: what is built, newest first, with every deviation and deferral.

Docs change in the same pull request as the behaviour they describe. For the engine and SDK repos, the status file here is updated alongside their pull request.

## Shape

One installed product, two programs:

- **The window** (`isonomic-desktop`): a GPUI app with a rail of app icons down the left. First-party apps are Rust crates compiled into it and shown as tabs.
- **The engine** (`isonomic-capsule`): a headless background program that runs the user's Capsule (databases, files, models, keys, sync, backups). It keeps running when the window closes and is the same program a server node runs.

They talk over one local socket using one API (`isonomic-sdk`). Apps never touch a database, a key or the network directly.

## Areas

| Folder | Covers | Spec sections |
| --- | --- | --- |
| `lessons/` | Rules learned from the earlier build | 1 |
| `product/` | Product shape, process model, lifecycle | 2, 3 |
| `engine-supervisor/` | Engine layout, supervisor, hardware profiles, RAM gate | 4.1 to 4.3 |
| `engine-registry/` | Embedded state store | 4.4 |
| `engine-services/` | Bundled services, PostgreSQL | 4.5 |
| `engine-broker/` | Socket server, sessions, capability tokens | 4.6 |
| `engine-later/` | Vault, gateways, functions, sync, Cells, node | 4.7 to 4.9 |
| `sdk/` | The contract between window and engine | 5 |
| `window/` | The GPUI window | 6 |
| `data-identity-backups/` | Data model, identity, payments, backups | 7 to 9 |
| `security/` | Security model, stated honestly | 10 |
| `packaging/` | The universal app and bundled binaries | 11 |
| `verification/` | Tests and the one-command report | 12 |
| `plan/` | Milestones, gates, risks, open decisions | 13 to 15 |

## Decisions

| # | Decision | Choice |
| --- | --- | --- |
| 1 | UI | GPUI through Isonomic UI (a renamed soft fork of GPUI Kit). Only that library names GPUI versions |
| 2 | Shape | Window + engine. One installer, one icon |
| 3 | Apps | First-party apps are crates compiled into the window, shown as tabs. Third-party apps run as Views, never as native code |
| 4 | App API | One contract in `isonomic-sdk`, over a Unix socket. No connection strings, no ports, no environment variables handed to apps |
| 5 | Engine core | Carry over the supervisor, registry, probes, hardware profiles, RAM gate and model router from the earlier build. Rewrite nothing that already works |
| 6 | Services | PostgreSQL, a cache and queue service, object storage, a vector database (lazy), local model server (lazy). No reverse proxy |
| 7 | Service exposure | Unix sockets inside the Capsule directory where supported; loopback with a random port and a per-service secret where not |
| 8 | Keys | A workspace master key in the OS keychain, wrapped by a passkey-derived secret, plus a recovery kit. Account recovery and data recovery stay separate |
| 9 | Models | Local only by default. Remote providers are opt-in, keys live in the vault, every remote call is logged |
| 10 | Backups | Scheduled encrypted snapshots to any folder. The same format moves a Capsule between machines |
| 11 | Agents | Each agent runs in its own Cell, supervised by the engine |
| 12 | Platform | macOS first, Intel and Apple Silicon in one universal build. Signing and notarization at the end |
| 13 | Honesty | First-party apps are trusted code, like the window itself. Mechanical confinement is claimed only for Views and Cells |

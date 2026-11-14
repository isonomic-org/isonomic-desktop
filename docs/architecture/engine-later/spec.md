# Vault, gateways and later modules

Spec sections 4.7 to 4.9. None of this is in Milestone 0. The crate names are reserved.

## Vault and keys

- A random workspace master key is created at first run and held in the OS keychain. Everything encrypted at rest chains to it.
- A passkey-derived secret wraps a copy of the master key for restore on a new device. Several passkeys can each hold a wrap.
- A printable recovery kit holds another wrap. Where the passkey extension is unavailable, the kit is mandatory.
- The vault also stores model provider keys, connection tokens and MCP credentials. Apps receive handles, never values.
- A versioned wrap format with frozen test vectors ships before any real data depends on it. A bug here is unrecoverable by design.

## Three gateways

| Gateway | Does | Default |
| --- | --- | --- |
| Models | Routes each request: app's pinned model, privacy setting, local capability, then fallback. Meters tokens and cost per app | Local only |
| Connections | Typed connectors and MCP tools for the user's linked accounts; refreshes tokens | No accounts linked |
| Egress | Outbound HTTP to hosts an app's manifest names and the user approved | Denied |

The menu bar toggle for AI web access disables remote model calls globally and must cut a stream in progress.

## Later modules

- **Functions.** App logic that must run with the window closed, as WebAssembly components with time and memory limits.
- **Sync.** Stores collaborative documents and relays updates between tabs, devices and peers.
- **Archive.** See `data-identity-backups/`.
- **Cells.** One virtual machine per agent, no secrets inside, one wire to the engine. On machines without the required virtualization support, agents run as functions only.
- **Node.** Registration, heartbeats and contribution reporting for machines that serve the network.

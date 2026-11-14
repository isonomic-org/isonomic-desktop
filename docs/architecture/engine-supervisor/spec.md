# Engine layout, supervisor, hardware profiles

Spec sections 4.1 to 4.3.

## Layout of `isonomic-capsule`

```
crates/
  capsule-core     types, errors, paths, config, clock and hardware traits
  supervisor       service lifecycle, stack, probes, logs, profiles
  registry         embedded database: desired and observed state
  broker           socket server, sessions, capability tokens
  vault            keys, tokens, keychain
  model-gateway    provider trait, router, RAM gate, local and remote providers
  connections      accounts, token refresh, MCP client
  egress           the only outbound HTTP path
  functions        WebAssembly runtime for app logic
  sync             document store and update relay
  archive          checkpoint, snapshot, restore
  cells            Cell supervisor behind one backend trait
  packaging        bundled binaries, architecture checks
xtask/             fetch-sidecars, dist, report
bin/isonomic-capsule
```

Milestone 0 builds only `capsule-core`, `supervisor`, `registry`, `broker`, `packaging` and the xtask. The rest are named now so nothing lands in the wrong place.

## Supervisor (carried over)

- **Spawn.** Each service runs in its own process group with an allow-listed environment, stdin closed, output captured to a ring of recent lines and a rotating file. Arguments and config files are rendered from templates before every start.
- **Stack.** A dependency graph, not a sequence.

| Tier | Services | Starts |
| --- | --- | --- |
| 0 | Registry (in-process), broker | Immediately |
| 1 | PostgreSQL, cache and queue, object storage | Immediately, in parallel |
| 2 | Vector database, model workers | On first use; model workers only if the RAM gate admits |
| 3 | App functions, Cells | When the services they declare are healthy |

- **Health.** Port or socket check, then a protocol probe. Every 5 seconds while starting, every 30 seconds after, and immediately when a consumer reports an error.
- **Hung services.** Two failed steady-state probes with the process alive means kill and restart.
- **Recovery.** Backoff from 1 second doubling to 60. More than five restarts in ten minutes marks the service degraded, keeps its last 50 log lines, and offers report and retry. Dependents pause; they are not killed.
- **Shutdown.** Reverse order by tier, each service with its own signal and a 15 second grace before a forced stop.
- **Environment.** The engine builds each child's environment from an allow-list. Nothing is inherited.

## Hardware profiles and the RAM gate (carried over)

One profile, chosen from physical memory, is the single source of every tunable. Boundaries sit between nominal sizes: under 12 GiB is Low, under 24 GiB is Standard, otherwise High. A user override wins; an unreadable override triggers a fresh probe.

| | Low | Standard | High |
| --- | --- | --- | --- |
| Database buffers | 128 MB | 512 MB | 1 GB |
| Cache memory | 256 MB | 1 GB | 2 GB |
| Default local model | up to 4 GB | mid-size | user choice |
| Model idle unload | 5 min | 15 min | never |
| Apps kept loaded | 2 | 4 | unlimited |

The RAM gate refuses a model for either of two reasons, checked in this order: it exceeds the profile's ceiling, or there is not enough free memory right now. A refusal carries required bytes, available bytes and a readable reason, and points to a smaller model or a remote provider.

# Registry

Spec section 4.4.

An embedded database file holding:

- desired state: what should run, grants, schedules
- observed state: ports, sockets, restart counts, last healthy
- installed apps and their manifests
- the audit log index

The window never reads it directly.

## Rules

- **Persist, then publish.** Write the registry before notifying subscribers. This applies to every state change the window can observe.
- **Desired state lives in the engine** and is reconciled after any restart: services, grants, schedules, Cells. Whoever is asked for runtime state must own the desired state.

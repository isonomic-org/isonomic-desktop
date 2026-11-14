# Product shape and process model

Spec sections 2 and 3.

## The window

- **Rail.** Home at the top, installed apps in user order, system tabs at the bottom. Icons only, labels on hover, a status dot when an app needs attention.
- **System tabs.** Home (launcher, recents, search), Store, Activity (what each app did), Settings (Models, Connections, Backups, Identity and devices, Storage, General).
- **Tabs keep their state.** An app is built on first open and stays alive. On small machines the hardware profile caps how many stay loaded; the least recently used is unloaded and restores from the engine when reopened.
- **Any tab can pop out** into its own window in the same process.
- **Menu bar icon.** Status light, open, snapshot now, AI web access toggle, quit.
- **Closing the window hides it.** The window process stays resident for the menu bar icon. Only Quit ends it, and Quit asks the engine to stop in order unless the user chose to keep it running in the background.

## Processes

```
window process (isonomic-desktop)        engine process (isonomic-capsule)
  GPUI app, rail, menu bar icon            supervisor, registry, broker
  first-party app crates                   vault, gateways, sync, archive
        |                                        |
        +---- isonomic-sdk over a Unix socket ---+-- PostgreSQL
                                                 +-- cache and queue service
                                                 +-- object storage
                                                 +-- vector database (lazy)
                                                 +-- model workers (lazy)
                                                 +-- Cells (one per agent)
```

## Lifecycle

| Event | What happens |
| --- | --- |
| Login | A per-user launch agent starts the engine. Core services come up. No window opens |
| Open the app | The window starts, finds the socket, connects. If the engine is not running it starts it and waits |
| Second launch or a deep link | Forwarded to the running window, which focuses the right tab |
| Close the window | Hidden. Engine and menu bar icon continue |
| Quit | The window asks the engine to shut down: Cells, then app functions, then services in reverse dependency order, each with its own signal and deadline |
| Window crash | Engine unaffected. Nothing is lost because edits are persisted before they are confirmed. The window restarts with the same tabs |
| Engine crash | The launch agent restarts it. Child processes die with it (process group) and are restarted from desired state. The window shows reconnecting |
| Update | The window replaces itself and the engine binary; the engine restarts between a checkpoint and a migration so a failed update rolls back |

## Targets

- Core services healthy within 10 seconds of login.
- Window to first frame within 400 ms when the engine is warm.

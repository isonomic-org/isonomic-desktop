# Window: status

Newest first.

## 2026-11-15 · Logs view (part of #6)

**Built** (`shell::logs`)

- Clicking a service on Home replaces the list with that service's logs; Back returns. The lines come from `system.logs` (the last 50, what the supervisor keeps), fetched when the view opens and again whenever that service's status changes (a new state or another restart). No polling; the fetch runs on GPUI's executor.
- A degraded service says so above its last lines. A failed fetch keeps the lines on screen and says why they are old. A disconnect isn't a change: the lines stay until the engine is back and reports.
- The shell takes a `LogSource` beside the live watch; the engine client is the real one. Stock components only: the Back button is Isonomic UI's `Button`, the lines use the theme's monospace face. No approved design yet.

**Verified**

- Linux: clippy `-D warnings`, 20 tests with `--locked`. Two new headless tests run the real shell against the SDK's in-memory engine over the real client: open a service's logs, new lines only with the next status event, Back; and a degraded service's 50 kept lines.

**Deviations**

- GPUI's test scheduler refuses wakeups from another thread, and the in-memory engine runs on its own Tokio runtime. The tests relay on the test thread: they wait for the client to see the engine's state, publish it to the window, and answer each fetch before handing it over. The window itself is unchanged; only the test's log source blocks.

**Not done: Retry.** The `system` interface has no call to retry a degraded service, and the wire interface belongs to the SDK. Requested as isonomic-sdk #14 (`system.retry`); the engine side is ready (the supervisor's `reset`). #6 stays open for it.

## 2026-11-15 · On Isonomic UI (PR #9)

**Built**

- `ui` re-exports Isonomic UI, a sibling checkout pinned to the commit in `UI_REF` (`15f2f4c`), instead of GPUI Kit 0.9.1. No other crate changed: they reach the library only through `ui`, and `extern crate ui as gpui;` still resolves GPUI's macros (Isonomic UI rewrites macro paths only for crates that name it directly).
- CI checks Isonomic UI out with the org's `UI_DEPLOY_KEY` (`ui: true` in the shared workflow).
- The report prints the Isonomic UI revision and `UI_REF`.

**Verified**

- Linux: clippy `-D warnings` and 15 tests with `--locked`, including the two headless window tests.
- Intel Mac: the release window builds (8 minutes, dependencies reused) and the Milestone 0 report passes from the app (healthy 3.0 s after launch, PostgreSQL back in 1.4 s, nothing left after stop). Window program 24.4 MB, up from 21.6 MB.
- Launched as an app, the window shows the rail and "All services healthy" in Isonomic UI's theme (IBM Plex Sans); Quit from the app menu stops the engine and PostgreSQL cleanly.

**Deviations**

- None. The typeface and theme are Isonomic UI's defaults; no screen has an approved design yet.

## 2026-11-14 (evening) · Menu bar icon, close to hide, Quit (PR #3)

**Spike result: menu bar icon**

- GPUI has no status-item API. The window process owns an `NSStatusItem` through the `tray-icon` crate, pinned to `=0.26.1`, created on the main thread inside GPUI's run loop. No helper process is needed.
- Menu clicks arrive on the main thread and go to a GPUI foreground task over a channel; the same task updates the light from the engine link's live watch. Nothing polls and nothing blocks the foreground thread.
- The menu: a status line, Open Isonomic, Quit Isonomic. The light is green (healthy), amber (starting), red (degraded) or grey (stopped or connecting). Its colors live in `ui::tokens::light` because the status item is drawn outside GPUI and cannot read the theme.

**Lifecycle**

- Closing the window (close button or Cmd-W) hides the app. The engine keeps running. Launching the app again or clicking its Dock icon shows the window.
- Quit (menu bar, app menu or Cmd-Q) asks the engine to stop every service in order, waits until the engine disconnects (up to 20 s), then exits.

**Verified on the Intel test Mac**

- Builds with Rust 1.98 and only the Command Line Tools; clippy clean; 9 tests pass. CI green on Linux and macOS.
- In `Isonomic Dev.app`: close hid the app with the engine and database still running; launching again showed it; Quit from the app menu left no process behind and PostgreSQL shut down cleanly. The status item is on screen.

**Not verified**

- The light's colors as drawn and the status item's own Open and Quit clicks: no background tool can see or click a status item.

**Deviations**

- The app menu's first title is the bundle name macOS supplies (`Isonomic Dev` in the dev bundle), not the menu name the code sets.
- Quit always stops the engine. "Keep running in the background" is an open decision and not built.

**Not done**

- Single instance and deep links. The app bundle is a hand-made dev bundle; the universal unsigned app is next.

## 2026-11-14 (later) · Skeleton: `ui`, `engine-link`, `shell`, Home tab

- `ui`: the single re-export, pinned to `gpui-kit = "=0.9.1"`. No other crate names GPUI or GPUI Kit. Crates that use GPUI's macros add `extern crate ui as gpui;`.
- `engine-link`: computes the socket path with the rule shared through the SDK, starts the engine binary when nothing is listening (at most once per 10 seconds), and owns one reconnecting client on its own Tokio runtime.
- `shell`: the left rail, a tab host and the Home tab, redrawn from the link's watch.
- The rail shows the tab's first letter in place of an icon until the icon set exists.
- First real run on the Intel test Mac: the window draws the rail and the live database status, and follows a force-kill through restarting to healthy.

## 2026-11-14 · Not started

- Component library pin candidate: `gpui-kit` 0.9.1.

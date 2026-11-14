# Window: status

Newest first.

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

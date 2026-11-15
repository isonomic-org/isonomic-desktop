# Window: status

Newest first.

## 2026-11-15 (night) · Retry for a degraded service (#6)

- A degraded service's logs view has a Retry button that calls `system.retry` (SDK 7c488c2). While the engine answers, the button reads Retrying and can't be pressed again. A refusal, such as `not_degraded`, shows beside it. The outcome comes back through the service's next status events, like any other change, and so do its new lines. Nothing polls.
- Headless tests against the testkit: a retry that ends healthy removes the button and brings the new lines, and a refused retry says why (`crates/shell/tests/logs.rs`).
- This closes the "Not done: Retry" item in the logs entry below.
## 2026-11-15 (night) · The app host: first-party app crates as tabs

**Built** (`shell::apps`, `Shell::mount`, the window's startup)

- An app is a crate that implements the SDK's `isonomic_app::App` (manifest, `build`, and optional `open_link`, `search`, `on_unload`). The window does not define a second interface: it links app crates and registers each with `isonomic_app::register`. The list is `first_party()` in `bin/isonomic-desktop`, in rail order.
- Each registration is a rail entry. The app is built the first time its tab is opened: the shell calls `apps.grant` with the app's own manifest, scopes a client to the token, and hands the app an `AppContext` (that client, its settings, its attention signal). The tab then draws the app's root view and stays alive. A refused grant shows on the tab and is tried again the next time the tab is opened.
- The rail follows the app's attention signal: a dot, or a count. It wakes on the signal; nothing polls.
- Tested in a headless window against the SDK testkit with the SDK's smallest app (`hello-app`): the grant goes out once, every later call carries the app's token, the attention dot shows, and a refused grant recovers (`crates/shell/tests/apps.rs`). Nothing waits on the engine.
- `cargo run -p isonomic-desktop --features example-app` mounts that app in the real window, to look at by eye. Release builds mount no app until Chat lands.

**Deviations**

- `shell` depends on the SDK's `isonomic-app`, which depends on Isonomic UI. `shell` still names only `ui`; the two are the same checkout, so the types match.
- App settings are kept in memory (`MemorySettings`) until the engine stores settings: they are lost on quit.
- Not yet routed: deep links and global search to `open_link`/`search`, and unloading tabs under the hardware profile's cap (`on_unload`). The SDK hooks are there; the window has no deep-link handler, search box or profile cap yet.
- Monograms stand in for app icons on the rail, as for Home.
- Isonomic Chat has no crate yet (its repo holds only a README). Requested in isonomic-chat (see the "Worker reports" issue for the link).

## 2026-11-15 (evening) · Pins: SDK 7c488c2, Isonomic UI 4c912f6

- `SDK_REF` moves to the SDK's `main` (7c488c2: `system.retry`, the app crate, conformance, sql, blobs and events). `UI_REF` moves to 4c912f6, the commit every V1 repo now pins.
- Nothing broke: build, clippy and tests pass on Linux unchanged, and `Cargo.lock` does not change.
- The SDK's own `UI_REF` is still 5cdf134; here its app crate builds against 4c912f6, the one checkout of Isonomic UI.

## 2026-11-15 (later) · Start at login (#7)

**Built** (`engine_link::agent`, the menu bar, the window's startup)

- At launch the window writes `~/Library/LaunchAgents/com.isonomic.engine.plist` and loads it, before the link starts and before the event loop. The plist names the engine inside the app and pins `ISONOMIC_CAPSULE_DIR` (a launch agent doesn't get the user's shell environment). The engine's own log goes to `logs/engine.log` in the Capsule. A moved app gets its plist rewritten and reloaded.
- The agent starts the engine at login and restarts it after an unsuccessful exit only (`KeepAlive` › `SuccessfulExit` false), so Quit, which stops the engine cleanly, doesn't loop. A second engine started while one serves exits 0 (engine #21).
- When the window finds no engine, it starts it through the agent (`launchctl kickstart`), so that engine is one the agent will restart; with the agent off it spawns the binary as before.
- Menu bar: a Start at Login check item, on by default. Turning it off deletes the plist and is remembered in the Capsule (`settings/start-at-login-off`); the change runs off the foreground thread.

**Checked by hand on the Intel Mac** (default Capsule directory, `cargo xtask bundle --host-only`)

| Step | Result |
| --- | --- |
| First launch of a fresh build | Plist written and loaded; the engine runs under launchd. 27.7 s before the engine started: macOS's first-run check of new programs |
| Quit from the app menu | Engine stopped over the socket, exit 0; launchd leaves it stopped (one run) |
| Login, simulated by unloading and loading the agent | No window; both services healthy 1.8 s and 0.8 s after load (cold start 208 and 445 ms) |
| Open the app with the engine up | One engine; the window connects to it |
| `kill -9` on the engine | launchd restarts it in 1.3 s; healthy 5.1 s after the kill, with the services the old engine left behind stopped first (engine #23); the window reads "All services healthy" |

**Found on the way, fixed in the engine:** with the default Capsule directory the cache never started, because its config split "Application Support" in two (#22); and a killed engine's services kept running and blocked the next engine's (#23). The new logs view showed the cache's config error.

**Not checked by hand:** a real logout and login (simulated with `launchctl`), and clicking the Start at Login item (the status item is out of reach of the tools I can drive; the agent behind it is unit-tested). Omar: please log out and in once, and toggle the item off and on.

**Deviations**

- Only an engine inside an app bundle gets an agent: a build run from the source tree would point the agent into a build directory.
- Turning it off leaves this login session's job loaded, because unloading it would stop the engine under the open window. It's gone from the next login.
- The agent is set up on the main thread before the event loop starts (two `launchctl` calls): after that nothing blocks a frame.


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

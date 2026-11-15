# Window: status

Newest first.

## 2026-11-15 (late) · Isonomic Chat mounted

- The window mounts `isonomic_chat::Chat` (`com.isonomic.chat`), first in the rail after Home. It comes from the sibling checkout `../isonomic-chat`, pinned to the commit in `CHAT_REF`. The pin is 64dcd18 on Chat's `main` (its #11), which pins SDK 21ed50d like the window and asks for `agents` in its manifest.
- Headless test: Chat in the real shell against the SDK testkit is on the rail, gets its token and draws its own view (`bin/isonomic-desktop/tests/chat.rs`).
- **The `ui` package is renamed `desktop-ui`.** Chat's workspace has its own package named `ui`, and cargo can't write two path packages of the same name and version to one lockfile. Code still names the crate `ui`, through the dependency key, so nothing else changes.
- **The workspace applies Isonomic UI's patch for GPUI's macros** (`vendor/gpui-pre-macros`), as the UI, SDK and Chat workspaces do. Chat's extra components need it. This repo's own code builds and tests the same with it.
- **CI:** `ci.yml` passes `chat: true`, so the shared workflow checks out isonomic-chat at `CHAT_REF` with the `CHAT_DEPLOY_KEY` secret.

## 2026-11-15 (late) · Failure states: the window says why it can't reach the engine

- The engine link now publishes a problem in plain words, on a watch (`EngineLink::problems`). It covers two cases: the engine binary can't be started (missing or not allowed to run), and the engine exits with a failure before it answers, in which case the problem quotes its last log line. A successful connection clears it.
- The engine the window starts writes its standard error to `logs/engine.log` in the Capsule (appended), as the login agent's engine already did. Before, a crash at start left nothing behind.
- Home shows the problem under "Connecting to the engine" until the engine answers.
- Tested: a missing binary, and a stand-in engine that writes its reason and exits 3 (`crates/engine-link/tests/link.rs`). Home shows the problem and drops it on connect (`crates/shell/tests/window.rs`).

**Each failure state from the orchestrator's list, and where it stands:**

| State | What the window does | Tested |
| --- | --- | --- |
| Engine not running at launch | The link starts it; Home reads "Connecting to the engine" | `link.rs`, `real_engine.rs` |
| Engine missing, or dies on start | Home says why, with its last log line | this entry |
| Socket refused | The same as not running: the link retries and relaunches at most every 10 s | `link.rs` |
| Engine exits and restarts | Home goes back to Connecting and redraws when it returns; apps' streams end with `Disconnected` | `window.rs` |
| A service degraded | Its logs and a Retry button (#20) | `logs.rs` |
| Model missing or download failed | First run says so and offers Try again; Continue waits | `first_run.rs` |
| Below the memory floor | First run says how much is needed and can't finish | `first_run.rs` |
| An app's grant refused | The tab says so and tries again on the next open | `apps.rs` |
| An app whose build panics | **Not handled.** `App::build` runs on the UI thread, and GPUI has no way to recover from a panic there. Apps are first-party and tested headless | n/a |

## 2026-11-15 (late) · First run downloads the local model

- After the memory check, first run asks the engine for the model it recommends for this machine (`models.recommended`) and whether it is installed (`models.info`). If not, the person starts the download. Progress shows as a percentage and as GB received out of total, with a Cancel button. Continue needs the memory check and an installed model.
- The engine owns the download (`models.download`): closing the window leaves it running, and the next first run attaches to it. Cancel calls `models.download_cancel`, and the stream's last event (`cancelled`) puts the step back to Download. A failure shows the engine's reason with Try again, and so does a stream that ends without an outcome.
- Tests: the view with a scripted setup, driven event by event (progress, cancel, failure, retry, installed, then Continue), plus the stream that ends early. The adapter over the real client is tested against the testkit's downloads (`crates/shell/tests/first_run.rs`, `shell::downloads`).
- Sizes are optional: a model or a download whose size the engine doesn't know shows the bytes received so far, never a percentage. Progress without a total falls back to the recommendation's size.
- `SDK_REF` is de3e0da, isonomic-sdk #26's merge on the SDK's `main` (optional sizes, `not_downloading`).

## 2026-11-15 (late) · Deep links and search reach apps

- **Deep links.** The bundle registers the `isonomic` URL scheme (`CFBundleURLTypes`). GPUI hands opened URLs to the window outside the app's context. They are queued and passed to the shell once its window is open, and the window is shown. `isonomic://<name>/...` goes to the mounted app whose id ends in `.<name>` (`isonomic://chat/...` goes to `com.isonomic.chat`). The shell selects that tab and builds the app if needed. Then the app's `open_link` gets the URL, held until the app is built. A link no mounted app names is logged and dropped.
- **Search.** `Shell::search` asks every loaded app's `search` and returns the hits in rail order, each naming its app. Unloaded apps aren't asked, because they have nothing built to answer with.
- Tested with the SDK's hello-app against the testkit: a link before the app is built selects it and arrives once it's built, an unknown app's link isn't handled, and search returns the app's hit (`crates/shell/tests/apps.rs`). The plist test checks the scheme.
- **Not built:** a search box. Screens wait on Isonomic UI's QA, so search is reachable from code only, for now.
- **Not checked on a Mac:** opening an `isonomic://` link from another app. The one-command check is in the Worker reports.

## 2026-11-15 (late) · Agent sessions: SDK 21ed50d

- `SDK_REF` moves to 21ed50d: the agents interface, the testkit's in-memory agents host and the `agents.` conformance checks. The SDK pulls harness-core from isonomic-harness over ssh at rev `7a73ca6`. This repo doesn't name harness itself, so cargo builds the SDK's one copy. `ci.yml` passes `harness: true` to the shared workflow. CI checks harness out with the `HARNESS_DEPLOY_KEY` secret.
- `Live` carries the session schema from `welcome`. Home warns when the engine writes a schema version this window can't read. A newer revision of the same version still reads, and apps render unknown events generically. Unit-tested.
- Apps reach `sessions.*` through their own scoped client; the window doesn't proxy them. The app host needed no change, and Chat's `agents` capability comes from its manifest.
- **Not wired: approvals and questions.** `approvals.decide` needs a user-action token minted by the window. The SDK defines neither the call that mints one nor how an app receives one, so I requested both as isonomic-sdk #25. V1 Chat is plain streamed text with no tools, so this doesn't block Chat's first screens.
- Not yet: purging an app's sessions when it is removed (`sessions.purge { app }`). The window has no remove-app flow yet.

## 2026-11-15 (night) · Unloading app tabs under the profile's cap

- The hardware profile caps how many apps stay loaded, using the supervisor spec's numbers: `low` 2, `standard` 4, `high` no cap (`apps::loaded_cap`). The window asks `system.profile` once the engine is connected. Until it answers, nothing is unloaded.
- Opening an app past the cap unloads the least recently opened one, never the tab on screen. That app's `on_unload` runs, its view is dropped and its rail marker clears. Reopening builds it again with the token it already has, so there is no second `apps.grant`. Its data comes back from the engine, as the product spec says.
- Tested headless against the testkit with three small test apps on a `low` machine: the oldest is told and unloaded, reopening it unloads the next oldest, and there are three grants in all (`crates/shell/tests/unload.rs`).
- **Deviation:** an unrecognized profile name gets the `standard` cap.
## 2026-11-15 (night) · App settings kept in the Capsule

- An app's settings (`AppContext::settings`) now survive a quit. `engine_link::settings::FileSettings` keeps one JSON object per app at `settings/apps/<app id>.json` in the Capsule. Only ids made of lowercase letters, digits, dots and dashes become file names; any other id stays in memory.
- Everything is read once, before the event loop. After that, reads come from memory, and each change is written by a background thread to a temporary file that is then renamed over the old one, so an app setting a value never waits on the disk. A file that doesn't parse reads as empty and doesn't stop the window.
- Tested: settings survive a reopen and stay per app; unsafe ids and bad files are ignored.
- **Deviation:** the spec keeps shell state in the engine. The engine has no settings store yet, so this is the interim store. When the engine gets one, only `CapsuleSettings` in the window's startup changes; apps don't.
## 2026-11-15 (night) · First run: the Capsule and the memory check

**Built** (`engine_link::setup`, `shell::first_run`, the window's startup)

- Before the link starts, the window creates the Capsule directory if it is missing (user-only, `0700`), so the engine starts in it, and notes whether it was new. `settings/setup-done` in the Capsule records that first run has finished. This uses the same kind of marker as start at login.
- Until then the shell shows first run in place of the tabs. Its steps: the Capsule (created now, or found), then the memory check. The check uses `system.profile`, and continuing needs at least 16 GB of memory in the machine, the V1 floor for local models. Below the floor, first run cannot finish. A failed check can be tried again. Continue writes the marker on a background task, then the tabs appear.
- Headless tests in the real shell: memory at the floor finishes and shows the rail; 8 GB cannot finish; a failed check recovers. The real client reads the testkit's profile (`crates/shell/tests/first_run.rs`).

**Not built: the model download.** The SDK has no call to download a local model with progress, no "not installed" state and no recommended first model. Requested as isonomic-sdk #21. Until it lands, first run says the download isn't available in this build and lets the user continue once the memory check passes.

**Deviations**

- The floor is checked against total memory, not available memory: available memory changes from minute to minute, and the engine's memory gate refuses a model that won't fit when it is loaded.
- The tests script the profile, because the testkit's `system.profile` is fixed at 16 GiB (also asked for in isonomic-sdk #21).
- "Done" is a marker file in the Capsule, not engine state, because the engine has no settings store yet.

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

Older entries, from Milestone 0 to the V1 pin move: [history.md](history.md).

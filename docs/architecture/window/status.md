# Window: status

Newest first.

## 2026-11-16 · Problem reports, part 1: capture (design agreed 02:56 UTC)

- **New crate `reports`**, with no UI dependency:
  - **Report:** a JSON document, format 1, of kind `panic`, `hang`, `app_failed` or `app_error`. It records the component (the window, or an app's id and version); version, build and architecture; the SDK, Chat and engine pins; macOS version, hardware profile, memory and the Capsule's format; time and uptime; the message; the backtrace; the window log's last 50 lines; and the tab on screen.
  - **Scrubbing:** home paths become `~`, quoted text longer than 40 characters becomes `[text removed]`, and messages are cut at 300 characters. Backtraces keep function names and crate-relative places only (`shell/src/home.rs:212`).
  - **Storage:** `reports/pending/` in the Capsule. A report is deleted on Don't send, after 30 days, or beyond 20 kept.
  - **Window log:** `logs/window.log`, rotated at 1 MB, recording state and errors, never content.
- **Panics:** the panic hook writes the report before the process ends. A panic inside an app's own code is blamed on the app whose crate comes first in the backtrace; the window lists its app crates at startup.
- **Contained app failures:** the calls the window makes into an app (build, deep links, search, unload) run inside `reports::panic::contain`. A panic there is written as `app_failed`, the app's tab says it stopped working, and the window and the other tabs go on. Opening it again from the rail builds it again.
- **Hangs:** a watchdog writes a `hang` report after 5 s with no beat from the foreground thread, and adds `stuck_secs` when it recovers. There is no backtrace, as agreed.
- **Crash-loop guard:** each launch is counted; 30 s up or a clean Quit resets the count. After three early ends in a row, the window starts with no app mounted and Home says why. A self-check never starts safe.
- **Pins in a report:** `bin/isonomic-desktop/build.rs` embeds `SDK_REF`, `CHAT_REF`, the engine checkout's commit and the bundle's build number.
- **Tested:**
  - scrubbing, with a prompt planted in a panic message, home paths and long text;
  - store, retention and pruning;
  - the crash-loop guard;
  - log rotation and its tail;
  - backtrace cleaning and blaming an app's crate;
  - the hook writing a report, and a contained panic being `app_failed`;
  - one hang report per hang, with its length;
  - an app whose build panics is contained in the real shell.
- **Not yet:** the notice, the review sheet and the setting (part 2). The sender is held: nothing leaves the Mac.
- **Chat ed43f66:** `CHAT_REF` moves on the same SDK and harness. Chat's rail icon is now `chat` from the catalog, and a picked model carries to every new chat.
- **Vocabulary:** "app catalog", never "app store". The four spec lines and the window's crate list are fixed.

## 2026-11-16 · Version 0.9.0: final screens on Isonomic UI 2e3116b (SDK 439a069, Chat 9b58161)

- **Pins:** `SDK_REF` 439a069 brings Isonomic UI 2e3116b, the components through QA, and harness ae4f1a7. `CHAT_REF` 9b58161 is on the same SDK. They move together. The workspace version is 0.9.0.
- **Final layouts** use Isonomic UI's components as built, with no restyling and no custom widgets. They use its spacing scale, its type sizes and its theme colours, in both themes.
  - **Rail:** a ghost icon `Button` per tab, using the manifest's icon from Isonomic UI's catalog, with the app's initial when the catalog has none. The tooltip names each button, which is also its accessible label. A `Badge` (a dot or a count) shows when an app asks for attention.
  - **Home:**
    - a title, then a line on the engine with a `Spinner` while connecting;
    - an `Alert` for each of: the move-to-Applications notice, why the engine can't be reached (its own last words, from `engine.log`), and an unreadable session format;
    - services in a `GroupBox`, each with a status `Tag` and a Logs button.
  - **Logs:** a Home button back; for a degraded service, an `Alert` and a Retry button that shows it is loading.
  - **First run:**
    - a `Stepper` across Capsule, Memory and Local model, with a `GroupBox` per step;
    - a `Progress` bar for the download, indeterminate when the size is unknown;
    - an `Alert` when memory is below the floor;
    - the engine's own words when a download fails, its disk-space refusal included;
    - Continue at the foot;
    - a readable width (`tokens::content_width`, 640 px).
  - **Install a model:** the same parts. An app that is opening shows a `Spinner`, and one that couldn't open shows an `Alert`.
- **Keyboard:** the window starts focus on the selected rail button, or on first run while it shows. Tab walks the rail and every button, and Enter or Space presses it. Tested: Tab then Enter opens an app; first run is finished by keyboard.
- **Scrolling a focused control into view**, which QA couldn't cover: GPUI and Isonomic UI don't do it. With six apps in a 160 px window, the focused last button stayed at y 252, outside the rail. The rail now follows its focused button with a scroll handle. Tested, with the window's size and the rail's bounds.
- **Title bar:** the window is titled "Isonomic" (see below). How the title bar is drawn needs a look on the Mac.
- **One version:** `cargo xtask bundle` refuses to build when the engine's workspace version isn't the app's, so the two ship at one version.
- **Not checked here:** how the screens look. A person needs to look once on the Mac, in light and dark.
- Chat's manifest names the icon `message-circle`, which Isonomic UI's catalog doesn't have, so Chat's rail button shows its initial. Asked of Chat via the Worker reports.

## 2026-11-16 · The window has a title

- The main window opens titled "Isonomic" (`lifecycle::window_options`, used by the app). Before, it set no title, so macOS's title bar, Window menu and app switcher showed none.
- Tested: the options the app opens with carry the title and a size. GPUI's headless window keeps no title, so the title bar itself needs a look on the Mac.

## 2026-11-16 · Apps' window requests reach the window (SDK #30)

- The window implements the SDK's `WindowRequests`. Every app's `AppContext` is given the window's handler (`AppHost::with_requests`). An app calls `request` from its own code, outside the shell, so the handler only queues the request (`install::ShellRequests`). The shell serves the queue (`Shell::serve_requests`).
- `InstallModel { model }` shows that model's download, named for the app that asked (`Shell::install_model`), and answers `true`. A kind this window doesn't handle answers `false`. The app learns the outcome from the engine (`models.info`), as the SDK says.
- Tested headless: a request through the trait shows the download with the right model.
- `SDK_REF` is 12ddf2f (isonomic-sdk #30) and `CHAT_REF` is 865152a. They move together, both on harness 9a3809f, so cargo builds one harness-core. Chat sends `InstallModel` on `ModelUnavailable`, and from home's Install button when no chat model is installed.
- Tested end to end with Chat against the testkit: with no chat model installed, Chat's Install reaches the window through the SDK's request, and the window shows `chat-small`'s download (`bin/isonomic-desktop/tests/chat.rs`).

## 2026-11-16 · Installing a model an app asks for (window side)

- `Shell::install_model(model, app, source)` shows a download view for that one model in place of the tab on screen. It returns a future that resolves to `Installed`, `Declined` (the person said not now) or `Failed` (the engine couldn't say whether the model is installed). A model that is already installed resolves at once and nothing is shown.
- The download, cancel and retry behave as in first run: the engine owns the download, an unknown size shows the MB received, and a stream that ends without an outcome is a failure. `install::ModelSource` is the client in the window and a scripted source in tests.
- This is the window's half of an app asking for a model (isonomic-sdk #31, coming in isonomic-sdk #30 as `AppContext::request(WindowRequest::InstallModel { model })`). When that trait exists, the window implements it by calling `install_model`, and Chat's `ModelUnavailable` reaches it.
- Tested headless: a download that fails, is retried and installs; not now; and an already-installed model. Also tested: the move-to-Applications notice shows on Home.

## 2026-11-15 (late) · Chat 2981abb: models in the manifest

- `CHAT_REF` moves to 2981abb. It's the same SDK (c0d2bf7) and the same harness rev, so only the Chat pin moves.
- Chat's manifest now asks for `agents`, `sql` (schema `chat`) and `models` (`any: true`, local only). The window grants from the manifest. The headless Chat test mounts it through that grant against the testkit, and a refused grant shows on the tab.
- **Not wired yet:** Chat raises `ModelUnavailable` when a picked model isn't installed, but an app can't ask the window for anything. I requested a window request in `isonomic-app` (isonomic-sdk #31). The window will answer it with first run's download step.

## 2026-11-15 (late) · SDK c0d2bf7 and Chat 6e2b6dc; Isonomic UI only through the SDK

- `CHAT_REF` moves with it to 6e2b6dc on Chat's `main`, which pins the same SDK and the same harness rev, so cargo builds one harness-core. Chat now asks for `agents` and `sql` and declares the schema `chat`. The window grants from the manifest, and the headless Chat test mounts it through that grant against the testkit. A refused grant shows on the tab (`apps.rs`).
- `SDK_REF` moves to c0d2bf7: the new harness rev (f061211, through the SDK only), model install, and Isonomic UI's `component` and `assets` features forwarded by `isonomic-app` (isonomic-sdk #27, on by default).
- The `ui` crate re-exports `isonomic_app::ui` and no longer depends on isonomic-ui. This repo's `UI_REF` is gone; the SDK's is the one pin, and the shared workflow reads it from the SDK checkout (isonomic-org/.github #8). `../isonomic-ui` stays checked out for the GPUI macros patch and the SDK's path dependency.
- `cargo xtask report` prints the SDK's `UI_REF`.
## 2026-11-15 (late) · Isonomic Chat mounted

- The window mounts `isonomic_chat::Chat` (`com.isonomic.chat`), first in the rail after Home. It comes from the sibling checkout `../isonomic-chat`, pinned to the commit in `CHAT_REF`. The pin is 64dcd18 on Chat's `main` (its #11), which pins SDK 21ed50d like the window and asks for `agents` in its manifest.
- Headless test: Chat in the real shell against the SDK testkit is on the rail, gets its token and draws its own view (`bin/isonomic-desktop/tests/chat.rs`).
- **The `ui` package is renamed `desktop-ui`.** Chat's workspace has its own package named `ui`, and cargo can't write two path packages of the same name and version to one lockfile. Code still names the crate `ui`, through the dependency key, so nothing else changes.
- **The workspace applies Isonomic UI's patch for GPUI's macros** (`vendor/gpui-pre-macros`), as the UI, SDK and Chat workspaces do. Chat's extra components need it. This repo's own code builds and tests the same with it.
- **CI:** `ci.yml` passes `chat: true`, so the shared workflow checks out isonomic-chat at `CHAT_REF` with the `CHAT_DEPLOY_KEY` secret.
- **Debug info:** with Chat in the build, CI's runner ran out of disk. Debug builds now keep line tables for this repo's crates and no debug info for dependencies (`[profile.dev]` in `Cargo.toml`).

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

Older entries: [history.md](history.md).

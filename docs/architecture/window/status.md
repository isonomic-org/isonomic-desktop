# Window: status

Newest first.

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

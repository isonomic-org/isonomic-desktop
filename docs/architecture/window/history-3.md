# Window: status history, part 3

Newest first. Moved from `status.md` to keep each file under 20 KB. These entries are newer than [history-2.md](history-2.md); new entries go in `status.md`.

## 2026-11-16 · Chat 292b12d: design 0.2

- **`CHAT_REF`** moves to 292b12d. It brings Chat's shell and sidebar, home and composer, and thread to design 0.2 (Chat #25 to #27).
- **Pins:** Chat pins SDK 439a069, and the window stays on 90eb83f. Both use harness ae4f1a7 and Isonomic UI 2e3116b, so one copy of each is built and `Cargo.lock` doesn't change.
- **Keys:** Chat's search takes `cmd-k` inside its tab. The window binds only `cmd-q` and `cmd-w`, so nothing clashes.
- **Tested:** fmt, clippy and the full suite, including the Chat tab mounted in the shell.
## 2026-11-16 · Acknowledgements

- **Where:** the Isonomic menu's new first item, Acknowledgements, opens a dialog with the licence notices the app carries (`Contents/Resources/NOTICES.txt`, read off the foreground thread). It brings the window forward first.
- **Outside an app bundle,** as in a development build, the dialog says the file isn't there.
- **Tested:** the dialog shows the text, or says it is missing, and Done closes it.

## 2026-11-16 · First run: "Your Isonomic ID"

- First run gains a step after the memory check and before the local model. The engine creates the person's ID on the Capsule's first start, and the window asks for it (`Setup::identity`) and shows it. It appears in a monospace line with a copy button (Isonomic UI's `Clipboard`), and a plain sentence says what it is for and that its key stays on this Mac.
- If it can't be made, for example because the Mac's key store refused or is unavailable, an `Alert` gives the engine's reason in its own words, with Try again. The model waits, and Continue needs the ID.
- The stepper now reads Capsule, Memory, Isonomic ID, Local model. The Capsule is one line under the title rather than a group. The steps scroll while Continue stays in a footer, so it's always in view.
- The self-check prints the ID, and fails with the engine's reason when there's none.
- Tested headless: the ID is shown with its copy button; a refused key store shows its reason, the model waits, Continue does nothing, and Try again goes on to the model. Also tested: first run finished by keyboard, which now tabs past Copy.
- **Wired to `identity.get`** (isonomic-sdk #34, merged as 90eb83f; `SDK_REF` moves there): `first_run::identity_from` returns the `id` as the engine gives it, and an `identity_unavailable` refusal's message as written. Tested against the testkit's scripted identity (`set_id`, `refuse`, `recover`). `CHAT_REF` stays ed43f66, on the same harness (ae4f1a7) and Isonomic UI (2e3116b). Refusal texts in tests say "the key store", as the SDK's fixtures do.
- **The problem-report notice gets the ID:** once connected, the window reads it with the same call and hands it over (`Shell::report_identity`), so the notice's checkbox appears. A refusal leaves it out.

## 2026-11-16 · Problem reports, part 2: the notice, the review and the setting

- **Notice** (an Isonomic UI dialog), for each kept report, oldest first:
  - shown at launch, after first run, for what an earlier run left (a crash or a hang), and straight away after a contained app failure;
  - titled "Isonomic quit unexpectedly", "Isonomic stopped responding" or "<App> stopped working";
  - one sentence on what a report is for, and "This report is kept on this Mac. Nothing leaves it.";
  - buttons **Show report**, **Don't send** (deletes it, then the next is asked about) and **Keep on this Mac**. Closing it any other way also keeps it. A kept report isn't asked about again in the same run.
- **Send is not offered:** this build has no destination and no sender (part 3 is held).
- **Show report** opens exactly the bytes that would be sent, the pretty-printed JSON with its log tail, in a second dialog over the notice. **Deviation:** the design says a sheet; a stacked dialog keeps the notice under it and returns to it with Back.
- **Isonomic ID checkbox,** off on every notice: "Include my Isonomic ID so you can contact me about this report". Ticked, the ID is added to the bytes shown; it is never written to a kept file. It appears once the shell is given the ID (`Shell::report_identity`).
- **Setting:** "Problem reports" on Home, with Ask each time (the default), Send automatically and Never send, kept in the Capsule's `settings/reports.json` with the destination, which is unset.
  - **Never send** deletes kept reports quietly at the next look.
  - **Send automatically,** with nowhere to send, keeps them and asks nothing.
- **No path in the shell:** the shell is handed a `reports::kept::Kept` and does every disk call on the background executor.
- **Self-check:** shows no notice, so nothing covers what it clicks.
- **Not yet:** a hang that recovers is asked about at the next launch, not when it recovers. The watchdog runs off the foreground thread and the window has no signal from it yet.
- **Tested:** the notice at launch, Show report's bytes, Don't send, Keep, Never send, Send automatically, the setting being kept, the checkbox, and the notice after a contained failure titled with the app's name.

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
- **Not yet:** the sender is held, so nothing leaves the Mac.
- **Chat ed43f66:** `CHAT_REF` moves on the same SDK and harness. Chat's rail icon is now `chat` from the catalog, and a picked model carries to every new chat.
- **Vocabulary:** "app catalog", never "app store". The four spec lines and the window's crate list are fixed.

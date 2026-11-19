# Window: status, history 4

Entries moved from `status.md` on 2026-11-18, newest first. Older entries are in `history-3.md`, `history-2.md` and `history.md`.

## 2026-11-16 · Problem reports, part 3: the sender (Omar agreed to sending; the host isn't chosen)

- **Where:** one URL from the build's configuration. A release compiles in `ISONOMIC_REPORTS_URL`, and optionally `ISONOMIC_REPORTS_KEY`, sent as `Authorization: Bearer`. A test build may set the same names when it runs, and those win. It must be `https://`; plain `http://` goes only to this Mac's loopback. With no URL, nothing is offered and nothing is sent, as before. `settings/reports.json` no longer has a `destination`.
- **What:** an HTTPS `POST` of exactly the bytes Show report displays: the kept document, plus its `id`, plus `isonomic_id` only when the box was ticked. Both are added outside the scrubbed fields. The id is also in `Idempotency-Key`.
  - The id is the SHA-256 of the Capsule's id and the kept file's name: stable for one report, and not linkable across reports.
  - It switches to the SDK's helper once format 1 carries an `id`.
- **Behaviour:**
  - Ask each time stays the default.
  - Send posts once, and the person sees "Sent" or "Couldn't send; kept on this Mac".
  - After a network error or a 5xx, the window tries again quietly: 30 s, doubling to an hour, for a day.
  - A 4xx is never tried again.
  - A 2xx deletes the kept file.
  - Send automatically posts every kept report quietly, never with the person's ID.
- **Its own runtime:** the sender runs on a one-thread runtime of its own (`reports::send::HttpSender`), not the engine link's. It uses the HTTPS client the dependency tree already built, with the system's root certificates.
- **The setting** reads "Reports you send go to <host>." once there is a destination.
- **The receiver's contract:** [report-receiver.md](report-receiver.md). It covers the method, the headers, the document, the answers, the 64 KiB limit, repeats by `id`, and to answer 503, not 429.
- **Never logged:** a body. The window log gets status codes and error kinds only.
- **Deviations:**
  - Retries live in the running window. After a quit, an unsent report is asked about again (Ask each time) or sent again (Send automatically) at the next launch.
  - The person sees only the first attempt's outcome; a later success is quiet.
- **Pins:** `CHAT_REF` is f928be5 (search shows when each chat was last active), on the same SDK, 72aca92.
- **Tested:**
  - against a local server on the loopback: the request line, the exact body, `Content-Type`, `Authorization` and `Idempotency-Key`; 201 sent; 400 refused; 503 and no server worth retrying; an oversized body refused before sending;
  - which URLs are allowed; the wait sequence and giving up after a day; ids stable for one report and different between reports;
  - in the shell: Send posts what Show report shows and deletes the file; a network error, then success after the wait; a 4xx never retried; Send automatically sends quietly without the person's ID.
## 2026-11-16 · The rail and first run after design 0.2

Reference: isonomic-chat `docs/design/v0.2/screens/app-shell.md` (the collapsed rail) and `first-run-and-system-states.md` (onboarding).

- **Rail:** the design's collapsed rail. It is 56 wide, with the sidebar fill and a 1 px right border, 12 above and below, 8 at the sides, and 4 between its buttons. Each button is a 28 px ghost icon button (`tokens::rail_button`). It was already 56 wide, filled and bordered; the buttons were the default size, 8 apart.
- **First run:** the design's onboarding page.
  - On the muted page fill (the sidebar colour): the "Isonomic" wordmark, then the stepper, then one card holding the steps, 16 apart and centred.
  - The card is 600 wide (`tokens::card_width`) and white, with a 1 px border, a 16 radius, 28 padding, 14 between its children and a medium shadow.
  - The Capsule line now carries the intro sentence.
- **Deviations, recorded:**
  - **One card, every step:** the design draws one step per card. Here all four stay in the one card, so the person keeps seeing their ID while the model downloads. The design has no screen for this window's steps.
  - **No card title:** "Welcome to Isonomic" repeated the wordmark right above it. Dropping it keeps the tallest state (a download in progress) above the footer.
  - **Continue stays in the pinned footer,** not the card's own button row as drawn. The steps can be taller than the window, and Continue must never scroll away (the build 96 fix).
  - **The wordmark is the interface sans,** as Chat's is. Isonomic UI has no display serif token.
  - **Home:** the design has no frame for the window's Home, so it is unchanged.
- **Tested:**
  - the wordmark and card are shown;
  - at the window's opening size (1100 × 720), the tallest step's Cancel clears the pinned footer (at the opening size it ends 24 px above the footer);
  - the rail's keyboard and scroll tests pass at the new size.

## 2026-11-16 · SDK 72aca92 and Chat 561e853: the design options

- **Pins:**
  - `SDK_REF` 72aca92 brings Isonomic UI 4f1207b, with the options Chat needs to match the design.
  - `CHAT_REF` 561e853 applies them: focusable sidebar rows, the comfortable composer with round Send and Stop, the search palette and the model picker.
  - They move together. One copy each of Isonomic UI, the harness and `isonomic-reports` is built.
- **The library's three breaking changes don't reach the window.** It builds no `PaletteCommand`, matches no `PopupMenuItem::Item` by its fields, and has no impl of its own on `IconName`. Nothing in the window changed.
- **Tested:** fmt, clippy and the full suite, including the Chat tab and the self-check's keys against Chat's shell.

## 2026-11-16 · The self-check presses Chat's shortcuts

- **What it presses:** after the prompt check (turn, stop, continue), the self-check sends Chat's design 0.2 shortcuts through the window's own key dispatch (`dispatch_keystroke`), as a keyboard would:
  - the design's sidebar toggle (`cmd-.`), then Chat's second toggle (`cmd-b`) to put it back. On macOS, `cmd-.` also means cancel;
  - from a new chat (`cmd-shift-o`), the next chat (`alt-down`), which should open the first chat in the sidebar.
- **What it prints:**
  - `keys: cmd-. toggled the sidebar: yes|no`
  - `keys: cmd-b toggled the sidebar back: yes|no`
  - `keys: alt-down opened the next chat: yes|no`
  Neither changes PASS or FAIL. Each key has 5 s to show its effect.
- **No clash:** the window binds only `cmd-q` and `cmd-w`. Chat binds `cmd-shift-o`, `cmd-.`, `cmd-b`, `cmd-k`, `alt-up` and `alt-down` in its own key context.
- **Pins:** `SDK_REF` aea8ba1 (Isonomic UI 66e93d3, with the run status bar aligned to the design, and the reports crate unchanged from fedba7d) and `CHAT_REF` 37f034c move together. Chat 37f034c is its design audit (58fe2d3) plus first run without a restart, versions on a real engine and the second sidebar key.
- **Code:** `self_check.rs` became `self_check/mod.rs`, with the shortcuts in `self_check/keys.rs`.
- **Tested headless:** against Chat's shell with its sample chats, both keys work through the window's dispatch.

## 2026-11-16 · Problem reports on the SDK's shared crate (SDK fedba7d)

- **`isonomic-reports`:** the window's problem reports now use the crate the engine uses too: format 1, the scrubber, and the backtrace helpers.
  - **Deleted here:** this repo's `scrub.rs`, `frames` and `short_path`, and the format types. The `reports` crate re-exports the shared ones, so call sites didn't change.
  - **Kept here:** the store, the setting, the panic hook and containment, the hang watchdog, the crash-loop guard and the window log.
  - **Renamed:** `System.macos` is now `os_version`. Format 1's key stays `system.macos`.
- **The scrubber, from fedba7d:**
  - quoted text of any length becomes `[text removed]`;
  - a person's ID in either form becomes `[id removed]`;
  - key-like runs become `[key removed]`.
- **The Isonomic ID:** when the notice's box is ticked, the window adds the ID as a top-level `isonomic_id`, outside the scrubbed fields. It is never in a kept file.
- **`WindowRequest::Report`:** an app's `AppContext::report(Problem { summary, detail })` is queued like `InstallModel`. The shell writes it as an `app_error`, with the app's id and its manifest's version, through the same writer as the panic hook, off the foreground thread. Then it shows the notice. A self-check has no reports, so it drops them.
- **Pins:** `SDK_REF` is fedba7d (Isonomic UI b4c9670 and the harness unchanged since abe8fe4). `CHAT_REF` stays fa3886f, which builds against this SDK checkout.
- **Tested:** an app's report going through the window's queue, written as `app_error` with a home path turned to `~` and a token removed, and asked about. The existing report, notice and hook tests pass on the shared crate.

## 2026-11-16 · The engine's problem reports, and the key rule

- **The engine's reports reach the same notice.** The engine writes format 1 into the same `reports/pending/` (isonomic-capsule #61). Its titles:
  - `panic` from `engine`: "The Isonomic engine quit unexpectedly";
  - `service_failed`: "The database stopped working", "The cache stopped working", or "The <name> service stopped working";
  - `model_server_failed`: "The local model stopped working".
- **When they're asked about:**
  - at launch, like the window's own reports;
  - while the window is open: at once when the engine's status shows a degraded service, and again 3 s (`tokens::report_rescan_delay`) after any status change (`Shell::review_on_status`).
- **Why the second look:** the engine writes its report after its own watch sees the failure, so the file can follow the status. A review is one directory read and never asks about a report twice.
- **No file-system watch:** it would add a dependency to cover what the second look already does.
- **Scrubber rule from the engine:** a run of 24 or more key characters with at least 6 digits becomes `[key removed]`, covering tokens, digests and the person's ID. The rule and its test cases are the engine's, word for word, so the two scrubbers match until both move to the SDK's shared crate.
- **Pins:** `SDK_REF` abe8fe4 (Isonomic UI b4c9670) and `CHAT_REF` fa3886f move together, so one copy of Isonomic UI is built. Chat brings its run status bar, no attaching, newest-first list order, prompt editing, and Try again with versions.
- **Tested:**
  - the titles and the scrubber's key cases;
  - a `service_failed` written while open, asked about when the service turns degraded;
  - a report written after the status change, asked about at the second look.

Older entries: [history-3.md](history-3.md), then [history-2.md](history-2.md), then [history.md](history.md).

# Window: status

Newest first.

## 2026-11-16 · The window opens at 1440 × 900 and remembers its size; the model named as Chat names it

- **Opening size:** the design's 1440 × 900 (`tokens::window_size`), fitted to the screen's visible area when that is smaller. On the 13-inch Mac that's about 1440 × 875. At 1100 × 720, Chat's 720-wide column nearly filled the content area.
- **Remembered:** the size the person leaves the window at is kept in the Capsule's `settings/window.json`, written off the foreground thread when the size changes, and used at the next launch, still fitted to the screen.
- **First run names the model as Chat does:** with `conversation_model::display_name`, so "Qwen3.5 2B", without the quantisation tag.
- **Pins:** `SDK_REF` 8f21cb6 (Isonomic UI 527d955) and `CHAT_REF` 99f310a (every QA id) move together.

## 2026-11-16 · Icons draw; first run reads as progress; the full pass explains itself

- **No icon drew, in every build so far.** The window created its app without Isonomic UI's asset source (`ui::application()` with no `.with_assets(..)`). Every icon was an SVG with no file behind it, so its space was drawn empty: the rail's buttons, the sidebar rows, Send, and the actions under a reply. The app now registers `ui::assets::AllAssets`.
  - **Every self-check now checks it:** the app's asset source must serve a known icon, else the check fails at once.
  - **The check build also counts pixels:** it counts the pixels the rail's Home icon draws, and fails at zero.
  - **Fonts weren't affected:** `ui::init` registers them.
- **First run:**
  - **The stepper:** only finished steps are filled, each with a check mark. The step being worked on and the ones after it are not, so it reads as progress instead of four filled circles.
  - **The Isonomic ID** is shortened in the middle (16 characters, an ellipsis, then the last 8), so it stays inside the card. The copy button beside it copies the whole ID.
  - **The model** is named as the engine names it for a person (`models.list`'s `name`, through the new `Setup::model_name`), not by its id, falling back to the id.
- **The full pass:**
  - **Step 10 opens the picker safely.** It looks for the picker's trigger before clicking it. The test helper's `click` panics on an id it can't find, which ended the run on the Mac with no line for steps 10 and 11. A missing id is now that step's FAIL, with its reason.
  - **The relaunch gets its state again:** it was missing only because that run died first.
- **Pins:** `SDK_REF` a3ac3bf and `CHAT_REF` 90ddc05 (Chat's wordmark by the brand rule, the sans greeting, and `chat-row-menu-<session id>`) move together.
- **Tested:** the window's asset source serves the icons; the ID shortening.

## 2026-11-16 · The wordmark in the sans: the brand rule

- **The rule (Omar):** no serif wordmark. "Isonomic" is set in the sans, Regular, and a product word after it in the sans, Light.
- **First run** now draws Isonomic UI's `Wordmark` component (`ui::extra::wordmark::Wordmark::new()`), at 20 px (`tokens::wordmark_size`). The window no longer names the serif anywhere.
- **Nowhere else** does the window draw the name. The window title and the Isonomic menu are the system's own text.
- **`crates/ui`** re-exports the SDK's `extra` (`isonomic-app` with the `extra` feature).
- **Pins:** `SDK_REF` is a3ac3bf (Isonomic UI a1b30e1, with the `Wordmark` component and the sans in Light), moved together with Chat's commit on it.

## 2026-11-16 · The wordmark in the serif; the recovering engine's message

- **Pins:** `SDK_REF` 1f28327 (Isonomic UI 0a8f39d, which bundles the serif) and `CHAT_REF` 8262fc8 move together.
- **First run's wordmark** is drawn in the theme's `serif_font_family`, at 20 / 28 and medium weight, the bundled serif's heaviest, as Chat's sidebar draws its own. It was in the interface sans, a deviation recorded at the time because there was no serif.
- **The set-aside notice follows the engine's new message** (isonomic-capsule #73). The window recognises it by its opening, "The list of apps and chats could not be read", and still by the older opening from an engine before it. It shows the message as it stands, with its counts of chats recovered and approvals kept.
  - Home's title for it is now "Isonomic recovered what it could". "Apps must be approved again" is no longer always true, and the engine's message says which.
- **Tested:** the recovering engine's message with its counts is shown as it stands.

## 2026-11-16 · The engine's set-aside notice, and its standard error moves

- **When the engine sets the registry aside:** a damaged list of approved apps is set aside at start (isonomic-capsule #72). Home then shows the engine's message as a plain warning, not an error, titled "Apps must be approved again". The text is the engine's own: "… Apps must be approved again. Their data and files are kept; chats from before are in the set-aside copy and are not shown."
- **How the window learns it:** the engine says it only in its log. Once connected, the window reads the latest run's lines in `logs/engine.log` (and `engine.log.1`, for a rotated log), off the foreground thread (`engine_notices`).
- **Shown once:** each incident appears once. The message names its timestamped set-aside file, so `settings/seen-engine-notices` remembers what has been shown.
- **Home's notices have their own titles:** they were all titled "Move Isonomic to Applications", including the crash-loop guard's, which now reads "Isonomic opened without its apps". Several can show at once.
- **Standard error moves:** the engine now writes its own `logs/engine.log` once it accepts the Capsule. The engine's standard error goes to `logs/engine.stderr`, from the window (`EngineBinary`) and from the login agent's plist (`StandardErrorPath`). An installed agent is rewritten at the next launch, since its plist's content changed.
- **The window's "engine stopped" message** quotes the last line of whichever of the two files was written last.
- **The gate and its report** read both the engine's standard error and the Capsule's `logs/engine.log`.
- **Tested:** only the latest run's message counts; each incident is shown once, and a new one is shown again; the engine-link tests pass with the new path.

## 2026-11-16 · SDK 4947e43 and Chat 5692f70: each report has its own id

- **Pins:** `SDK_REF` 4947e43 (Isonomic UI ecd21d1) and `CHAT_REF` 5692f70 (Chat's last design rows) move together. The window reads no `PaletteCommand::icon`, the one change that isn't additive.
- **The id:**
  - A report written now carries its own random `id`, from the shared crate's `Report::new`. The sender sends that id, in the document and in `Idempotency-Key`.
  - A report kept from before gets `send::derived_id`: the first 128 bits of the SHA-256 of the Capsule's id and the file's name without `.json`. That's the same 32 hex form as the SDK's ids.
  - The SHA-256 stays, rather than the SDK helper's FNV-1a, so two reports from one Mac share nothing a receiver could link.
  - The receiver's contract says 32 hex characters.
- **Tested:**
  - a report's own id is used;
  - an older one gets a derived id in the SDK's form;
  - derived ids are stable for one report and differ between reports.

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

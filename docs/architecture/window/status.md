# Window: status

Newest first.

Older entries: `history-4.md`, then `history-3.md`, `history-2.md` and `history.md`.

## 2026-11-18 · The menu bar icon is the mark

- **The icon:** the Isonomic mark, black on clear, as a template image (`assets/icon/menu-bar-mark.png`, 36 pixels for 18 points), so the system colours it for a light or dark bar. It replaces the coloured status dot.
- **Status:** no longer shown by the icon. It stays as the first line of the icon's menu and as its tooltip.

## 2026-11-18 · Onboarding is its own window; its first two screens

- **Design:** "Onboarding 0.4", "01 · App launch" and "02 · Get started" (Isonomic Desktop, nodes 372:1247 and 448:4).
- **The mark's sound:** the brand's recording (`assets/onboarding/isonomic-logo-sound.wav`, 4.2 s) plays with the mark's intro only: it starts with the intro and fades out over 0.25 s so it is silent as the mark leaves the centre (1.75 s; the recording's own tail runs to 4.2 s). It plays through the system's sound player on macOS (`onboarding/sound.rs`). Other systems and the tests' windows are silent. Nothing is re-timed to it.
- **A window apart from the app's** (`shell::onboarding`): fixed at 1280 × 800, all page, rounded corners, no title bar, and nothing to close, shrink or grow it by. The page itself drags it. A new person sees it first; the app's window opens when they choose "Start new account". A self-check goes straight to the app's window.
- **One motion, from one clock:** the mark plays the brand intro in the centre (drawn natively from the brand file's keyframes, `ui::logo`), turns once and shrinks into its corner; the laptop somersaults up from the bottom edge as its display lights; the title and the two buttons come up from the bottom edge in turn.
- **The laptop is 3D while it moves** (`crates/device3d`, new): the 13-inch MacBook Pro's model from the devices repository, drawn on the processor a frame at a time off the foreground thread, by the stills' own camera, so the landed laptop lines up with the design's picture, which then fades in over it. Half detail while it turns fast, full as it lands; 75 to 132 frames in the 1.5 s flight on the Intel Mac.
- **Places, sizes, colours and timings** are in `ui::onboarding`; assets in `assets/onboarding/`.
- **Deviations:**
  - Every timing after the brand intro is ours: the design has two still frames and Omar's description of the motion.
  - "Recover your account" does nothing yet: its screens aren't built. "Start new account" opens the app's window on the earlier first run.
  - Light only; the 2020 13-inch model for every Mac, as the design draws it.
- **Checked:** on the Intel Mac in a test build on a throwaway Capsule: the settled window beside the design's frame, and frames of the motion captured from the window.

## 2026-11-16 · SDK a6276aa and Chat 60b8fbd: the quiet composer

- **Pins:** `SDK_REF` a6276aa (Isonomic UI 2b8a583) and `CHAT_REF` 60b8fbd move together. Chat's composers now focus quietly, with no blue, and are 96 high when empty. Search reads "Esc", and the four-point star sits by the greeting.
- **The quiet focus look:** Isonomic UI offers it on the prompt composer only (`set_quiet_focus`). The window draws no composer or text input of its own; all of them are Chat's. So nothing in the window takes it.

## 2026-11-16 · Chat d413d78

- **Pins:** `CHAT_REF` d413d78 (each reply keeps its own text state; deletes hold across reloads; `loads_finished()`). `SDK_REF` stays at 8f21cb6. See verification's step 13.

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


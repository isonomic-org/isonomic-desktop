# Window: status

Newest first.

Older entries: `history-6.md`, then `history-5.md`, `history-4.md`, `history-3.md`, `history-2.md` and `history.md`.

## 2026-11-25 · Onboarding works over the engine

- **One thing to ask.** Everything real is asked of `onboarding::backend::Backend`: the terms kept as accepted, the Isonomic ID, its recovery kit, the local models and their downloads, whether Isonomic starts at login, and the choices kept at the end. The window's is `engine::EngineBackend`, over the shell's client, the Capsule's settings and the login agent; `backend::Sample` answers with the design's samples, for tests and the headless pictures. `onboarding::open` takes the backend.
- **Pins.** SDK `123d0b4` (Isonomic UI `6e0c4a5`) and Chat `67150eb`; a release needs the engine at `07f1831` or later, for the recovery kit's calls. The self-check's delete step now answers Chat's new "Delete chat?" dialog.
- **Accept and continue** keeps `terms { version, accepted_at }` in the Capsule's settings, under `isonomic`.
- **Continue with Face ID** asks the engine for the ID (`identity.get`, which makes it on a new Capsule). The sheet stays up its time and until the ID is here. A refusal takes the sheet down and is said between the step's two buttons, in the engine's own words, and the press can be made again. No passkey is made and Face ID is not asked for: the sheet is drawn, and the keys are the engine's.
- **Your account exists.** shows the ID, and "Copy" copies it.
- **Save your recovery kit** asks for the kit when the step comes on (`identity.kit_export`, `person_asked: true`) and shows its 24 words. "Save kit file" asks where and writes the kit's document, which never holds the words (`Isonomic Recovery Kit.json`). "Continue" waits for the kit, its file and the tick. "Print" is gone, as approved for V1. The words are never logged and never put on the pasteboard; they are let go with the window.
- **Confirm your recovery kit** checks the three words against the real ones, then tells the engine (`identity.kit_saved`) before the holder closes. A refusal holds the step.
- **Turn on local AI models.** The engine lists only the models a Mac has, so the list is the one it would start with (`models.recommended`) and whatever is installed, each with its size and licence. Under the engine's memory floor a note says so and no switch can be put on. A model switched on is fetched by "Setting up your workspace".
- **Integrations** are kept as choices; nothing connects yet.
- **Finish.** "Start Isonomic at login" is the login agent's switch, and stays off where a build has no agent (run from a source tree). The other two switches are kept as choices only.
- **Setting up your workspace** keeps the choices (`onboarding` in the settings), starts each download, and follows the first to its end, with how far it is under the pill and "Continue while it downloads" (the download is the engine's). A failed download is said for four seconds, then onboarding goes on; Chat offers the model again.
- **Calls never block a frame.** An answer waits only on the engine, so it is polled on the foreground thread; what blocks on a settings file or the login agent runs on a thread of its own (`backend::off_thread`), and the kit's file is written on the background threads ahead of everything else. Those threads are busy with frames: in turn behind them, saving the kit's file waited over three seconds in the development build. A call gives up after 30 s.
- **Words changed from the design, because the real kit made them untrue:** the kit step's lead and its note (the words and the file together restore the ID; keep them apart), the confirm step's note (a written copy, not a printed one), the ID's caption and the line under it, and the kit's small print.
- **Still the design's words, and not true yet:** the passkey, the secure enclave and its syncing; the smart account; the four "workspace root" rows; "This Mac · Touch ID"; the Finish step's notes on what starts. "Why no password?", "Add another passkey", "Import model", "Custom integration", "Learn more" and "Recover existing account" do nothing: the first five have nothing behind them, and recovery (`identity.kit_restore`) has no screens designed.
- **Checked.** `tests/onboarding_backend.rs`: the ID made, a refused ID said and tried again, a kit that does not come, a refused `kit_saved`, the choices kept and the models fetched, a Mac short of memory. Run on the Intel Mac against the engine at `07f1831` on a scratch Capsule with its keys in memory: the ID made, the kit exported and its file written, the words confirmed, the recommended model (1.3 GB) downloading with its progress said. The login agent was not touched. Pictures of the refusals: `ONBOARDING_SHOTS_TROUBLE=1` with `ONBOARDING_SHOTS`.
- **Not yet:** the holder's engraved letters are the model's, not the kit's; the engine has no list of models to install, so a new Mac is offered one.

## 2026-11-25 · A piece turns the way it goes; no hairline round the moving laptop

- **Turning.** A piece that goes left or right turns about its upright line, that way; one that goes up or down turns head over heels. Was: the laptop went head over heels on its way left to the sidebar and right to the middle, the pen tumbled end over end as it came and went sideways, and the ID card turned about its upright line as it lifted. Now: the laptop turns left on its way to the sidebar and right on its way to the middle (`laptop::Flip`; still head over heels as it comes up from the bottom edge), the pen turns about its upright line, and the ID card flips over backward as it lifts, then slides straight down into the wallet (`flow::Slip::flat`). The wallet, the seal, the clipboard, the holder, the drives and the pills already turned sideways as they went sideways. Going back plays each the other way.
- **The hairline.** While the laptop moved, a hairline showed along the right and bottom edges of its picture. Moving, it is drawn at half the detail and stretched, and a stretched picture's outermost pixels take a little of whatever lies beside it in the window's memory. Its picture is now shown a point short all round (`laptop::shown`); the canvas is clear there. Seen in captures of the release build before; not in the one moving frame caught after.
- **A frame's picture is let go once.** With frames kept, the same picture can come twice running (two moments on one tick, a piece at rest); it was let go from the window's memory while still shown. `stage::put` lets the last one go only when the new one is another picture.
- **device3d: depth in perspective.** Depth was blended evenly across a triangle, which is wrong in perspective: turned about its upright line, the laptop's display showed a black parallelogram, the lid behind it coming through the middle of the display's two large triangles. The inverse of depth is blended now (`render::fill_band`), which is exact for a flat face.

## 2026-11-25 · The mark's intro keeps time with its sound; 3D frames are kept and drawn ahead

- **The mark and its sound.** Was: the intro started on the first frame, but the recording opens with 0.3 s of quiet, so the four locked 0.3 s before the chord; and the sound was faded out at 1.55 s and stopped at 1.8 s of its 4.2 s, as the mark left. Now: the sound plays at its own speed to its own end, the intro starts with its first note (`at::MARK_STARTS`), so the lock lands on the chord, and the mark stays in the centre while the chord rings and leaves at 3.0 s (`at::MARK_LEAVES`). The intro's own keyframes are untouched. Everything after moves later by the same 1.2 s.
- **Going back no longer leaves a piece showing.** A piece that has gone back under its panel isn't drawn (`flow_view::TUCKED`): the ID card and the wallet, the drives and the pills, the seal, the clipboard and the pen, the holder and the padlock. Before, the panel then slid away and left it sitting there.
- **Frames are kept.** `stage::frame` keeps what it draws, by what was drawn (model, camera, poses, size, light), up to 384 MB, oldest out first, and forgets it all when onboarding closes. Every loop asks for frames on the screen's ticks (`stage::snap`, sixty a second), so a moment asked for twice is the same frame: a move that has played plays backward, or again, without being drawn.
- **Frames are drawn ahead.** While the mark plays, the laptop's flight and the terms' coming on are drawn (`terms::warm`); when the terms rest, the signing (`signing::warm`); while it signs, the seal's flight (`Seal::warm`). A press on "Start new account" before its frames are there waits for them, 0.6 s at most.
- **Measured on the 2020 Intel MacBook Pro, development build:** the first screens' 270 frames are drawn ahead in 3.7 s; the laptop's flight showed 65 of its 99 frames, 54 before. The development build draws the window itself unoptimised, so it is slower than what ships; the test app can be opened from the release build (`UI-Test/open-release.sh`).
- **Not yet:** the kit's holder is drawn as it plays, not ahead (its frames are kept once drawn); frames are swapped by a timer, not by the screen's own refresh.

## 2026-11-25 · Onboarding's edge cases, and a second design pass

- **"Save your recovery kit"** can't be left forward until the kit is ticked as stored: "Continue" is disabled and `next_step` holds (`Onboarding::held_back`, `store_kit`).
- **The words typed back** may have capitals and spaces around them, as the screen says (`kit_run::fits`). A wrong word is red whether or not it has the keyboard, and takes the keyboard so it can be put right. With nothing typed, the line under the grid asks for the words instead of calling word 4 wrong.
- **The lists** show what their tab and their search box ask for (`steps::listed`, `MODEL_KINDS`, `LINK_KINDS`), and say so when that is nothing.
- **"Setting up your workspace"** cuts a long name short with an ellipsis.
- **Tested in `tests/onboarding.rs`:** "Cancel" on the Face ID sheet and asking again; the unticked kit; words with capitals and spaces; a wrong word, then put right.
- **Design pass:** the notes are a grey card with the 20 px "i" centred beside the text (laid out here with the library's icon, because the library's alert pins its icon to the top at its own size); the kit's grid has the design's 187-wide cells with 12 between; "Learn more" is underlined only when pointed at; button icons are 20; the holder is the design's size (181 tall, open) with bright faces and a lighter frame; a step with no rule starts 22 under its heading.
- **Not done:** "Recover existing account" still leads nowhere, since recovery has no screens; Escape doesn't dismiss the Face ID sheet; the primary colour is the library theme's near-black where the design has navy.

## 2026-11-25 · "Decline" and the Face ID screen's "Previous step" play backward

- **Was:** "Decline" on the terms and "Previous step" on "Create your account with Face ID" showed the screen before at once.
- **Now:** both play what brought the screen on backward (`onboarding::rewind`). Everything on those screens is placed by the third screen's clock, so going back is that clock running backward: `Onboarding::terms_now` reads it from the rewind while one runs, the 3D pieces are drawn again for each time it passes, and what it goes back past is put back (`Onboarding::passed`). No button is heard until it lands.
- **"Decline":** the clipboard and the pen go back under the content, the stepper drops, the content slides off to the right, the laptop turns back to the middle, and the title, the pill and the link come back.
- **"Previous step":** the seal goes back under its panel and the panel slides off; the clipboard and the pen come back in from the left; then the signing is undone four times as fast as it was done (`signs::UNDO_RATE`): the pages lift, the signature is unwritten, the pages settle. The terms can be accepted again.
- **Also:** "Learn more" in the two lists is the library's `Link`; the Face ID sheet has the design's sizes; the shell's onboarding panels have no raw sizes left (the spacing scale, or `ui::onboarding` tokens); the kit's words are typed into their inputs in `tests/onboarding.rs`.
- **Checked how:** the tests land both; `ONBOARDING_SHOTS_BACK=1` with `ONBOARDING_SHOTS` draws both and shows each lands on the right screen. The headless window runs a rewind to its end in one step, so the motion itself is still to be watched in the window.
- **Known, in the library:** buttons, switches, checkboxes and tabs show the arrow, not the pointing hand, when pointed at; only links show the hand. `Link` is always underlined, where the design's "Learn more" is not.

## 2026-11-25 · Onboarding's controls are Isonomic UI's, and a design pass

- **Was:** onboarding drew its own buttons, switches, tabs, search boxes, checkbox, badges, notes and the three boxes the kit's words are typed back in, and several had no icon.
- **Now:** every control is the library's (`ui::component`): `Button` (with the design's icons: arrows, plus, check, printer, download, add, fingerprint), `Switch`, `TabBar`, `Input`, `Checkbox`, `Tag`, `Alert`, `Spinner` and `Link`. What is still drawn here is layout and art only: cards, rows, the stepper, the Face ID sheet's frame, and the 3D pieces.
- **The words typed back** are three `Input`s. Letters only, twelve at most (`kit_run::fits`); Tab and Enter are the input's own. The view keeps what is typed (`Onboarding::word_event`) and writes it back to the inputs when it is filled in for a test or held after confirming.
- **"Get started"** follows the design as it is now: the laptop is 300 across at (490, 150), the title is under it, one pill ("Start new account"), and "Recover existing account" is a link under the pill.
- **"Setting up your workspace"** shows this Mac's own name (`scutil --get ComputerName`, or "This Mac"), and its pill has the design's mark and a spinner.
- **The stepper** has six steps, as the design has: the recovery kit's two screens are one step.
- **Smaller things from the pass:** card padding counts the border; the address card has no gaps between its lines; "Active" is the pale green badge with a dot; notes are the grey card, not the blue one; the words not asked for are faint; list headers have the add button beside the title and the tabs and search under it.
- **How it was checked:** `ONBOARDING_SHOTS=<folder> cargo test -p shell --test onboarding_shots` draws every screen at rest to a PNG without a window on screen, for laying beside the design. The timer-driven 3D pieces are not in those pictures.
- **Differs from the design, on purpose:** the tabs are the library's segmented tabs (grey bar, white chosen tab) where the design draws an outlined bar; the spinner is the library's.
- **Not yet:** raw sizes in the shell's onboarding panels still to move into `ui::onboarding` tokens.

## 2026-11-24 · "Previous step" plays a step's coming on backward

- **Was:** "Previous step" on the later steps showed the step before at once, at rest.
- **Now:** the step being left plays its coming on backward (`flow::Flow::back`): its art turns over and goes back under its panel, the panel slides off to the right, the stepper's words change back, and the step before's art comes back in from the left edge. The step before is there when that ends (`Onboarding::land`), and no button is heard until then.
- **What each step's art does backward:** the ID card comes out of the wallet and lies over it again; the holder's tiles turn back, it closes and goes under the panel; on "Confirm your recovery kit" it turns to face the person again.
- **The locked kit:** past "Confirm your recovery kit" the kit stays locked. Going back to that step shows it closed with its padlock on, and "Continue" goes on without asking for the words again. Going back out of it unlocks it first: the padlock's body drops, it goes back under the panel, and the holder opens with its back turned, before the rest plays backward.
- **From "Your account exists."** back to "Create your account with Face ID" likewise; the seal comes back in from the left edge.
- **Not yet:** "Previous step" on "Create your account with Face ID" and "Decline" on the terms still go back at once.

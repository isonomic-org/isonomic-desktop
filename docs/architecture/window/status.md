# Window: status

Newest first.

Older entries: `history-5.md`, then `history-4.md`, then `history-3.md`, `history-2.md` and `history.md`.

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

## 2026-11-24 · Onboarding ends in Chat, not in the old first run

- **Was:** when onboarding handed over, the app's window opened on the old first run (Capsule, Memory, Isonomic ID, Local model), which onboarding replaces.
- **Now:** when onboarding was shown, the app's window marks the Capsule as set up and opens Chat (`isonomic://chat/home`) as soon as the engine is connected. The old first run is not shown after onboarding.
- **Still there:** the old first run's code, for the self-check alone, which has no onboarding and drives it by its ids. It goes when the self-check drives onboarding instead.
- **Not done by anything now:** the memory check against V1's floor and the first model's download were the old first run's. Onboarding's own steps have nothing behind them yet, so a model is downloaded only when an app asks the window for one (`Shell::install_model`).

## 2026-11-24 · The recovery kit's holder, its padlock, and the words typed back

- **What it is** (`shell::onboarding::kit`, `kit_run`, `kit_view`; sizes and timings in `ui::onboarding::kit`): the devices repository's titanium holder and padlock, drawn in 3D, in place of the design's two flat pictures. The holder is two plates on a hinge with the kit's 24 words on letter tiles (their first four letters, as the panel has them).
- **"Save your recovery kit":** the holder comes out from under the panel closed, turning over as every step's art does; it opens; then its tiles lift and turn into place one after another. "Continue" waits for it to open.
- **"Confirm your recovery kit":** the holder turns its back, so its words can't be read. The panel is the design's grid ("08b"): the 24 words, with 4, 11 and 19 to type back. A click on one of the three, or the step coming on, gives them the keyboard; Tab moves between them and Return presses "Continue".
- **A wrong word** shakes the holder, rings its cell in red and says which word it was. Capital letters and the spaces around a word don't matter.
- **The right three:** the holder closes with its mark up and its loops toward the person, the padlock comes out from under the panel, its body snaps up onto its shackle, and after a moment they turn over and leave by the left edge as "Turn on local AI models" comes on.
- **Drawn at the screen's own pace:** a thread of their own draws the holder's and the padlock's frames (`kit_run::Easel`). The window asks for one at a draw, when how they lie has changed, and takes it at a later draw, so a frame waits on no timer and on no free moment of the foreground thread.
- **Ours, not the design's or the model's:** the mark's floor is shaded so that it shows (the file gives it the plate's own colour); the padlock lies flat under the loops, its shackle over their hole, not through it.
- **First pass:** the typed words have no selection, no pasting and no moving the caret; the holder and the padlock leave by being drawn narrower, as the other steps' art does, not by turning in 3D.

## 2026-11-18 · The ID card lies over the wallet, then tucks into it

- **"Your account exists."** had the wallet on the left and the card beside it. The wallet is now in the middle of the sidebar and the card lies over it, its top out past the wallet's mouth, not in it.
- **Coming on:** the wallet comes out from under the panel first and the card follows it (`steps::at::CARD_FOLLOWS`).
- **On "Next step":** the card turns over as it lifts clear of the wallet's mouth, goes in between the wallet's back and its pocket, a little past where it rests, and comes back; a moment later the two turn over and leave by the left edge, and then the next panel comes on (`steps::at::CARD_LIFTS`, `CARD_TUCKS`, `CARD_RESTS`).

## 2026-11-18 · The lists show the design's marks

- **"Turn on local AI models" and "Integrations and connected apps":** each row showed its first letter on a chip. It now shows the design's own mark, in the design's 40-point box with 16 after it (`shell::onboarding::logos`; `ui::onboarding::steps::LOGO`).
- **The pictures** are the design's, exported at three pixels to a point into `assets/onboarding/logos`.

## 2026-11-18 · The drives and the pills carry their marks

- **"Turn on local AI models" and "Integrations and connected apps"** now draw the devices repository's final models: each drive with its mark engraved in its top, each pill with its mark embossed. Only the model files changed; they are drawn as before.
- **Still not the design's:** the drives have a cap at the back as well as the front, and the window draws no shadow under them.

## 2026-11-18 · The drives are tilted as the design has them

- **"Turn on local AI models":** the drives were drawn from straight above. They are now drawn as the design draws them, the way an icon of a drive is: from in front and 51° up, so the top shows with the front edge and its light under it (`ui::onboarding::steps::DRIVE_ELEVATION`).
- **Nothing narrows toward the back:** `device3d::Camera::axonometric` is the stills' camera from two hundred model diagonals away, so edges that are parallel on the drive are parallel in the window.
- **Where they rest** is where the design's drives are, without their shadows (`steps::DRIVES`).
- **Not the design's yet:** the models have a cap at the back as well as the front, and the window draws no shadow under them.

## 2026-11-18 · The wallet and the ID card are 3D

- **"Your account exists."** now shows the devices repository's card wallet and ID card, each drawn once from above, in place of the design's flat badge. The card rests beside the wallet, outside it.
- **On leaving the step**, the card first goes up over the wallet's mouth and slips in, between the wallet's back and its pocket (the wallet is two sprites for that); then they turn over and leave by the left edge together, and only then does the next panel come on (`steps::at::SLIP`).
- **Where they rest** is ours: the design has the card already in the wallet.

## 2026-11-18 · Every later onboarding screen, as a first pass

- **What there is now** (`shell::onboarding::flow`, `flow_view`, `flow_bodies`, `steps`, `art`; tokens in `ui::onboarding::steps`): from "Continue with Face ID" the Face ID sheet comes up and goes, then six steps follow, each over the last: "Your account exists.", "Save your recovery kit", "Confirm your recovery kit", "Turn on local AI models", "Integrations and connected apps" and "Isonomic lives in your menu bar". "Finish setup" brings "Setting up your workspace", and then onboarding hands over to the app's window.
- **Nothing is behind any of it.** No Face ID is asked for; no account, kit, model, integration or setting is created, switched on, connected or saved. The address, the 24 words, the lists and the dates are the design's samples. The switches and the connect buttons flip; "Copy", "Print", "Save as PDF", "Add another passkey", "Learn more", the tabs, the search boxes and the three word fields do nothing.
- **Every step comes on the same way:** the last step's art turns over and leaves by the left edge, the next panel slides over the last from the right, the stepper's words and bars change, and the step's art turns over as it comes out from under the panel. "Previous step" goes back without a motion.
- **The art:** the drives and the pills are the devices repository's 3D models, each drawn once from above; the badge, the recovery kit's holder and seal, and the menu bar are the design's flat pictures until they have models. A sprite turns over by being drawn narrower, not by its model being drawn again. The lists show a letter where the design has each model's or integration's mark.
- **"Setting up your workspace":** everything clears and the laptop, drawn in 3D, turns over on its way to the middle.
- **The laptop's top is at 150 on every screen** (it was 163 on "Get started", and the design has it at 235 on "Setting up your workspace"): only where it is across the window changes. The name and the pill under it on "Setting up your workspace" are moved up with it.
- **Not from the design yet:** the Face ID sheet's glyph and spinner, the recovery flow ("Recover your account"), the failed Face ID screen, and the typed confirmation of the kit's words.
- **Owed:** many of the new panels' sizes are written where they are used, not in `ui::onboarding`; the model and integration names are the design's copy.

## 2026-11-18 · Onboarding's fourth screen, "Create your account with Face ID"

- **Design:** "Onboarding 0.4", "04 · Passkey ceremony — idle" (Isonomic Desktop, node 372:1249). Sizes and timings are `ui::onboarding::passkey`; logic and the seal are `shell::onboarding::passkey`, drawing `passkey_view`.
- **It follows the signing**, where onboarding used to hand over: the next panel slides over the last from the right, the stepper's words change, and the Face ID seal turns over as it comes out from under the panel into the sidebar. Then the dotted line comes back and two rings of light breathe around the seal.
- **The seal is 3D:** the devices repository's Face ID tile, drawn by `device3d` from above. Its rings of light are see-through in the model, which `device3d` doesn't draw, so the window draws them, breathing as the model's own `pulse` animation does (one breath in 2 s, the wider ring 0.14 of a breath behind).
- **The buttons:** "Continue with Face ID" hands over to the app's window; no Face ID is asked for yet, and the later screens aren't built. "Previous step" goes back to the terms as they were before they were accepted, without a motion of its own. "Why no password?" does nothing yet.
- **Every later step should come on the same way:** its panel over the last from the right, then its props out from under it.

## 2026-11-18 · The laptop is its 3D model on every screen

- **No picture:** the landed laptop used to give way to the design's finished picture, which showed as a change in its shading the moment it came to rest. The picture and its asset are gone. The laptop rests as its last drawn frame, in full detail, on the second screen and in the sidebar. Earlier entries that mention the picture describe what was.
- **"Decline"** draws the laptop again at its size in the centre.
- **The handover no longer kills the app:** at the end of the signing the onboarding window was closed and the app's opened in the same turn. On a Mac with a Touch Bar the system then raised an exception over the closed window it was still watching, and the app died. The app's window now opens first and onboarding's closes 0.3 s later (`at::CLOSES_AFTER`). Seen on the 13-inch Mac: the app's window is up after the signing and no crash report is written.

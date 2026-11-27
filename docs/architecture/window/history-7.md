# Window: status, earlier entries (7)

Moved from `status.md` on 2026-11-27. Newest first. Older still: `history-6.md`.

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

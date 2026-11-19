# Window: status

Newest first.

Older entries: `history-5.md`, then `history-4.md`, then `history-3.md`, `history-2.md` and `history.md`.

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

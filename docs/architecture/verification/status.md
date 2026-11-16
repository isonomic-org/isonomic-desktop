# Verification: status

Newest first.

## 2026-11-16 · The full pass through Chat's ids; the relaunch's count explained

- **Steps through Chat 99f310a's ids** (Isonomic UI 527d955, stable ids):
  - **Step 6:** pin, rename and delete go through each row's menu, `chat-row-menu-<id>` and its `-pin`, `-rename` and `-delete` items.
  - **Step 7:** it reads the found row's last-active text from `command-<id>-meta` before Return.
  - **Step 10:** it opens the picker by its trigger, `("model-picker", "trigger")`, checks that the list (`model_picker::list_id`) is open, and checks that the picked model's row (`model_picker::row_id`) is the chosen one.
- **Step 9's "2 chats, not 3" was the script, not a lost chat:**
  - Chat's `is_loaded()` stays true after the first load, so step 6's wait after `reload()` passed on the old list.
  - The reload's answer, read while the third chat still existed, then landed after the delete, and put that chat back in the sidebar's memory only.
  - The relaunch read Chat's store, which had the two chats it should.
  - Step 6 now waits for the reloaded list itself, which Chat replaces whole, before deleting.
  - **A small race in Chat this exposes:** a reload answered after a delete shows the deleted chat again until the next reload. Passed to isonomic-chat.
- **The problem-report notice now opens:** the shell shows it from its next draw, and nothing asked for one. The step refreshes the window, and is now step 12 with a line of its own.
- **Screenshots:**
  - the model picker open;
  - Acknowledgements, now light and dark;
  - the problem-report notice;
  - the running status bar, which is the thread mid-answer shot.
- **Tested headless:** steps 4–7, 10 and 12 against the SDK's test engine, plus the count a relaunch reads: 2 in memory and 2 in the store after the steps.

## 2026-11-16 · The icon check runs after first run, and is proven with real pixels

- **Why `--full` failed on the Mac (0.9.0-155):** at once, with "no rail-home on screen". The check build's pixel count ran as soon as the engine connected, while first run was on screen and there was no rail yet. No step ran.
- **Now the icon check has two halves:**
  - At connect, every self-check checks that the app's asset source serves a known icon, as before.
  - The pixel count runs at the end of `ready`: after first run finishes, or at once when first run was already done.
- **The pixel count measures the icon now:**
  - It counts only the central half of the button, where the icon sits.
  - Ink is the pixels that differ from that area's commonest colour.
  - Before, the selected Home button's own fill counted as ink.
- **Proven for real, headless:** the new check-build test `render_tests` uses GPUI's headless renderer. That's Metal on macOS; on Linux, Mesa's software Vulkan driver (lavapipe), which this session has. It draws the real shell through first run, and:
  - while first run is up, the check can't run, which is the Mac's failure;
  - after first run, the Home icon draws 276 pixels;
  - with an asset source that has no icons, the same button draws 0.
  - Where no renderer starts, the test is skipped, not failed. Checked with Vulkan hidden.
- **Pins:** `CHAT_REF` is 5cedc14, Chat's display names without the quantisation tag, on the same SDK, a3ac3bf. Chat's main has since moved to a newer SDK (8f21cb6), which this pin doesn't take.

## 2026-11-16 · `cargo xtask self-check --full`: the function pass, with screenshots

- **How to run it:** `cargo xtask bundle --check-build`, then `cargo xtask self-check --models <dir> --full`. The short check stays the default. `--full` refuses a bundle that isn't the check build.
- **The check build:** the window's `check-build` feature turns on Isonomic UI's test support, which can click a control by its id and render the window to an image, plus PNG writing. The bundle is marked `Contents/Resources/check-build`. A release never builds it (`dist` sets it off).
- **Driven from inside the app:** through the window's own dispatch (`dispatch_keystroke`, `dispatch_action`, and clicks by id in the check build) and Chat's own entry points, never the system's mouse or keyboard. Each step prints `full NN name: PASS|FAIL (reason)`; a FAIL doesn't stop the steps after it, and any FAIL fails the run.
- **The steps, after the prompt, stop and continue steps:**
  - 4 Try again under turn one's reply: a second version, and the switcher reads 2 / 2, 1 / 2, 2 / 2.
  - 5 Edit turn one's prompt: select all, type, save. Version 3 of 3, with 1 / 3 and 3 / 3 reachable.
  - 6 The chat list: two more chats make three. Pin one, and it moves to the top. Rename one by typing, and the name holds after the list is read again. Delete one, and it's gone.
  - 7 Search: the palette opens, "Renamed" is typed, and Return opens the renamed chat.
  - 8 The keys, as before.
  - 9 The relaunch, in the link run on the same Capsule: no first run, the same ID, the same chats, and the first chat's turns. The prompt run prints `state id=… chats=… chat=… turns=…`, and `xtask` hands it to the link run (`ISONOMIC_SELF_CHECK_EXPECT`).
  - 10 The model picker on Chat's home: an installed model is picked, and the list opens by its trigger's id.
  - 11 Isonomic › Acknowledgements: the dialog opens, and `NOTICES.txt` has a section for every program in `Resources/sidecars`.
- **Screenshots:** `<scratch>/screens/NN-name.png`, rendered from the window's own last frame (`Window::render_to_image`), never a capture of the screen. They're taken light at the window's size, and again dark for the main screens: first run, a thread mid-answer, the status bar stopped and done, a finished thread with versions, the sidebar with a pinned chat, search, Chat's home, the model picker, Acknowledgements and a problem-report notice (a sample report kept in the scratch Capsule). `xtask` prints the folder at the end.
- **Deviations:**
  - Pin, rename's start and delete go through Chat's shell, not the row menu, and search's row text isn't read. Chat's sidebar rows, their menu and the palette's rows have no test ids, so these are asked of isonomic-chat.
  - Two status bar states never come up in a passing run, failed and reconnecting, so they aren't captured.
  - Turn 2's timing never fails the run.
- **Tested headless:** steps 4 to 7 run the same functions against real Chat in the real shell, on the SDK's test engine, and pass. That run found two faults before the Mac would have:
  - a new chat is first known by a provisional id;
  - the edit field opens only once the turn can be tried again.
  The relaunch, the picker list, Acknowledgements and the screenshots need the Mac.

## 2026-11-16 · The link run is a second launch again

- **Why it's back:** with the engine's isonomic-capsule #75, approving the same app again signs nothing, so a second engine on the same scratch Capsule can open Chat.
- **Now:** the prompt run gets a fresh scratch Capsule. The link run is a second launch on that same Capsule, as it was before #58, so "first run already done" and a relaunch with an existing ID are tested again.
- **Confirmed by the engine:** the link run's key check expects two `keys kept in memory: scratch Capsule` lines, one for each engine run on the Capsule.
- **Kept from #58:**
  - a failed tab fails the step at once, with its reason;
  - a failed run prints the last lines of the engine's and the window's logs.
- **Needs an engine at a6dc338 or later.** With an older engine, the link run fails at once with the engine's key-store refusal.

## 2026-11-16 · The link run fails on a second engine with in-memory keys: each run gets its own scratch Capsule

- **Build 143:** the link run failed twice with "timed out after 180s waiting for the app to open". Build 134 passed it.
- **The cause, from the code:**
  - Since #55, a scratch Capsule's Isonomic ID keys live only in its engine's memory.
  - The link run started a second engine on the same Capsule. The person's key log was there from the first run, but the keys were gone.
  - Opening Chat asks `apps.grant`. The engine records the approval, then gives the app its ID and grants, which needs the person's signing key (`Person::signer`). That key can't be opened, so `apps.grant` answers an error.
  - Chat's tab went to Failed with it, and `link_check` waited only for Open, so it waited out the step.
  - A person's real Capsule isn't affected: its keys stay in the computer's key store.
  - Not run here: the engine's identity crates need the private `isonomic-id` checkout.
- **Fixes:**
  - `cargo xtask self-check` gives each run its own fresh scratch Capsule. Model files are hard-linked where the disk allows, else copied. The link run therefore goes through first run again, with its own ID, and the second launch on one Capsule is no longer covered by this check.
  - `link_check` fails at once with the tab's reason, as `open` already did.
  - A failed run prints the last 20 lines of the scratch Capsule's `logs/engine.stderr`, `logs/engine.log` and `logs/window.log`, with home paths as `~`, so a failure on the Mac explains itself.
- **Softer, not addressed:** turn 2's first text (8.5 s and 32.9 s against 0.9 s) and turn 1's end (91 s and 98 s against 38 s). The Mac was busy with other apps; the next build will show whether it holds.

## 2026-11-16 · The self-check writes nothing to the keychain

- **Why:** each `cargo xtask self-check` made a fresh scratch Capsule, and the engine kept that Capsule's Isonomic ID keys in the person's login keychain. That's three items per run, never removed.
- **Now:**
  - The scratch Capsule's `capsule.json` is written as `"scratch": true` before the engine first opens it. The engine keeps the mark.
  - Every run sets `ISONOMIC_KEY_STORE=memory`, so the engine keeps the ID's keys in memory for the run (isonomic-capsule #70, which honours it only for a scratch Capsule).
- **Confirmed by the engine:**
  - After each run, the check reads the scratch Capsule's `logs/engine.log` for the engine's line, `keys kept in memory: scratch Capsule`: one for each run so far.
  - With it, the check prints `keys: memory (scratch Capsule), nothing written to the keychain`.
  - Without it, the run fails with `keys: FAIL, the engine did not confirm a memory key store`, followed by the engine's refusal if it logged one.
- **Not touched:** the items earlier runs left in the keychain. Removing them is Omar's call.
- **Tested:** the fresh Capsule is marked scratch, and the engine's log lines are read for confirmation or refusal.

Older entries: [history.md](history.md).

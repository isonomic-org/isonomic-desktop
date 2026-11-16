# Verification: status

Newest first.

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

## 2026-11-16 · Acceptance check from the disk image alone (Apple Silicon)

- `ISONOMIC_SELF_CHECK_DOWNLOAD=1` lets the self-check download first run's model when it isn't installed, as a person would. The download gets 30 minutes; other steps keep 3. So the check runs on a machine with only the disk image: no repos, no toolchain, no model file.
- **The release build on the Intel Mac passed the acceptance check** (build 96, both architectures):
  - Turn 1: first text 6.78 s, end 16.76 s.
  - Turn 2: first text 2.62 s, stopped with 5 characters.
  - Continue: first text 0.54 s, Done at 10.14 s.
  - Link: Chat received it.
  
  The Apple Silicon half needs an Apple Silicon Mac. The command is in the Worker reports.

## 2026-11-16 · Self-check PASS on the Intel Mac; `cargo xtask self-check` is the release acceptance check

- **Rerun with the waiting fix:** PASS, exit 0, no processes left. Build: desktop 4d7c0e1, capsule 3139266, SDK c0d2bf7, Chat 2981abb, the 2B model, a scratch Capsule.
  - The window was up, the engine connected at 0.30 s, first run passed and Chat mounted at 0.31 s.
  - Turn 1: first text 7.11 s, ended Done at 91.24 s.
  - Turn 2: first text 27.54 s, stopped at 27.55 s with 1 character.
  - The continue: first text 0.42 s, ended Done at 84.90 s with a real reply.
  - Stop and continue work end to end in the real window; the engine didn't refuse the continue.
- **Acceptance check:** `cargo xtask self-check --models <dir> [--capsule <dir>] [--prompt <text>]` runs it against `dist/Isonomic.app`.
  - It copies the models into a fresh scratch Capsule.
  - It runs the prompt check (first run, a Chat turn, stop and continue), then the link check (`isonomic://chat/home` as a launch argument).
  - It prints every `[self-check]` line and fails unless both runs pass.
  - It is the acceptance check for every release build.
- **Engine findings (isonomic-capsule):** the second turn's first text (27.5 s against 7.1 s) and the length of the first reply.

## 2026-11-16 · Self-check on the Intel Mac: a real Chat turn works; the continue check was wrong

- Ran on the MacBookPro15,1 with the 2B model in a scratch Capsule. The window was up and the engine connected at 0.06 s, first run passed and Chat mounted at 0.07 s. Turn 1 had its first text at 6.88 s and ended Done at 109.33 s with about 3,000 characters. Turn 2 had its first text at 30.51 s and was stopped at 30.52 s with 1 character. The continue then read 0.00 s, Stopped, and the check failed (exit 1). No processes were left behind.
- **The FAIL was the check's own mistake.** Continue is a request: Chat's `resume` sends `sessions.continue`, and the turn changes only when the engine's events arrive. The check read the turn straight after pressing Continue, saw the stopped turn (not open, one character), and took it for the end of the continued turn. It now waits for the turn to move on from the stop (its revision changes) before waiting for new text and the end. The decision is unit-tested with the Mac run's numbers.
- With Chat a3c87ff, a refused `sessions.continue` is kept on the turn (`Turn::refusal`), and the check prints its words: `FAIL: the engine refused to continue: …`.
- **Still to see on the Mac:** whether the continue then produces text. Whether the engine continues a reply stopped at its first token is the engine's to check (isonomic-capsule). The second turn's slow first text and the long first reply are on the engine side too.

## 2026-11-15 (late) · The window checks itself: `ISONOMIC_SELF_CHECK`

For a Mac nobody can click on, the real app drives itself and reports in text on standard error. It exits 0 (PASS) or 1 (FAIL).

- **`ISONOMIC_SELF_CHECK=<prompt>`**, with `ISONOMIC_CAPSULE_DIR` naming a scratch Capsule, runs these steps:
  1. The real window, engine and model start.
  2. It waits for the engine.
  3. First run goes ahead once its memory check passes and its model is installed.
  4. Chat opens. The prompt is typed into Chat's composer and sent with Enter, through Chat's own path, as a person would.
  5. It prints the reply and the times to first text and to the end of the turn.
  6. It sends the prompt again, stops it at the first text, continues it, and prints the same numbers.
  7. It stops the scratch engine and exits.
- **`ISONOMIC_SELF_CHECK=link`** with an `isonomic://` link as a launch argument opens the link inside the app, never through the system's URL handling. It passes when the named app is on screen and, for Chat, received the link.
- **It leaves the person's own setup alone:** no login agent is installed or changed, and there's no menu bar item. Run it as the binary inside the bundle, a second instance with its own Capsule. Never run it through `open` or the URL scheme.
- **Tested headless:** a draft followed by Enter starts a Chat conversation through Chat's composer (`bin/isonomic-desktop/tests/chat.rs`). The full check needs a model and a display, so it runs on the Mac.

## 2026-11-15 (late) · The window against the real engine: `cargo xtask e2e`

**Built:** `cargo xtask e2e` builds the engine from `../isonomic-capsule` and fetches its services for this machine. It then runs `crates/engine-link/tests/real_engine.rs` against them, in a fresh scratch Capsule. A plain `cargo test` skips that test.

**What it proves, against the real binary:**
- Nothing is listening, so the window's engine link starts the engine, then connects.
- `welcome` names the engine and its session schema.
- PostgreSQL does its first-time setup and reports healthy.
- `system.profile` and `models.recommended` answer.
- An app is granted from its manifest, and is refused what its manifest doesn't allow.
- Quit stops the engine.

**Run on Linux here as an ordinary user:** passed in 0.8 s after the build. The engine was isonomic-capsule aaa8e14 at SDK de3e0da. This machine has 15.7 GiB, so the engine answered `models.recommended` with `insufficient_memory` (the 16 GiB floor), which is correct.

**Not yet covered, and what it needs from the engine** (isonomic-capsule #45):
1. A stand-in model server program the engine runs in place of `llama-server` from the bundle directory, scripted to stream a known reply, so a turn, stop and continue can run without a model.
2. A test model catalogue (`ISONOMIC_MODEL_MANIFEST` or similar) whose source the test serves on `127.0.0.1`, so first run's download completes without the network.
3. The cache on Linux: it has no Linux build, so the stack reports it failed. The test only needs the database, but a usable window should say so in plain words (failure-states task).
4. A memory floor the test can lower or a profile override that `models.recommended` honours, so a runner with less than 16 GiB still gets a recommendation.

**CI:** not yet. It needs the shared workflow to check out isonomic-capsule and isonomic-inference next to this repo (inputs and deploy keys for this repo). The services download is 31 MB. The test's own run is under a second, and the engine build is the main cost.

**Rules:** the database refuses root, so `cargo xtask e2e` refuses to run as root.

## 2026-11-14 (night) · `cargo xtask report` (PR #4)

**Built**

- `cargo xtask report [--host-only] [--no-build]`. It builds the app (or reuses `dist/`), starts the app's engine against a scratch Capsule, and drives it through the SDK client exactly as the window does.
- Item 1: waits for healthy (up to 180 s: first start includes first-time database setup and macOS's one-off check of new programs).
- Item 2: `kill -9` on the postmaster from its pid file, then waits for PostgreSQL to be healthy with a higher restart count (up to 90 s). Passes only if the new postmaster has a new pid and none of the old one's workers are left.
- Item 3: `system.shutdown`, which is what Quit sends. Passes only if the engine exits 0, PostgreSQL's log shows a completed shutdown and its pid file is gone, the engine's log ends with `stopped` after the request, the socket is removed, and no engine or PostgreSQL process is left. Exited-but-unreaped processes count as gone.
- Item 4: cold start (the engine's own figure, plus time from launch to healthy) and the app's size, part by part.
- The block also carries the machine, toolchain, version and git revision of each repo, both `SDK_REF`s, the PostgreSQL version and the engine's lines saying where each binary resolved. On failure it adds the last 25 lines of the engine and PostgreSQL logs and keeps the scratch Capsule. It is saved to `dist/report.txt`.
- On Linux it runs the engine repo's release build with its fetched `sidecars/`; item 4 is n/a there.

**Verified**

- Linux, as a non-root user: items 1 to 3 pass (PostgreSQL back in 1.2 s, 0 of 8 old workers left; stopped in 0.1 s, 0 of 10 processes left). With the engine missing, item 1 fails, 2 and 3 are skipped and the scratch Capsule is kept.
- Intel test Mac: the universal app passes all four items. Runs:

| Run (Intel Mac) | Cold start (engine) | Healthy after launch | Recovery after kill | Stop |
| --- | --- | --- | --- | --- |
| Host-only app, first run of new binaries | 626 ms | 11.8 s | 1.9 s | 0.1 s |
| Universal app, first run of new binaries | 313 ms | 7.1 s | 1.5 s | 0.1 s |
| Universal app, quiet machine | 75 ms | 1.8 s | 1.4 s | 0.1 s |
| Universal app, another session running a release build | 1186 ms | 54.1 s | 3.0 s | 0.2 s |

"Healthy after launch" includes first-time database setup, which every run does because each report uses a fresh scratch Capsule. The first run of new binaries also waits for macOS's one-off check of programs it has not seen. Every timing here is on the Intel Mac; Apple Silicon is covered by CI only.

The pasted block is in `plan/status.md`.

**Not covered**

- The window's half of the gate (the rail, the light through kill and recovery, Quit from the menu) stays a check by eye. The report says so in its block.

## 2026-11-14

- **Engine:** the hermetic suite exists for what is built (51 tests) and runs in the build sandbox. No CI runs it yet.
- **Real-service check:** `pg_smoke`, a manual example in the supervisor crate, runs the engine side of the Milestone 0 gate against a real PostgreSQL. It is the seed of `cargo xtask report`.
- **Window:** nothing yet.
- **Report command:** not started.

# Settings: status

Newest first.

## 2026-11-27 · Every page, tab, sub-page and dialog of Settings 0.3

- **Built.** The 35 pages of the design, their 21 tabs and sub-pages and 61 dialogs, drawn from `assets/settings` by `shell::settings` with Isonomic UI's components only (`spec.md`). Light and dark.
- **In the window** behind `ISONOMIC_SETTINGS`: a gear at the foot of the rail and ⌘,. Without it the window is as it was. The pages hold the design's samples, so they are not offered to everyone yet.
- **Pins unchanged**: SDK `91c8d2a`, Isonomic UI `6e0c4a5`. Isonomic UI's `main` is one change ahead (two new components) and one change is open (the permission prompt); neither touches a component Settings uses.
- **Works**: the sidebar, its search (pages and settings by title; "No settings match"), a page's own search, tabs, sub-pages and back, every dialog and its close, switches, choices, segments and the companion's card, which are kept in the Capsule's settings under `isonomic` / `settings`. A row's bin takes the row off until Settings opens again.
- **Not connected**: everything else. A button with no dialog in the design, and every dialog's button but "Cancel" and "Done", says "This is not connected yet." Fields are not kept. The design's "Saved" note is not shown, since nothing is saved by those buttons.
- **Ready to connect**: Models and Plugins' servers through `shell::workspace` (the catalogue, downloads, removal, connections); Start at login through the login agent, as onboarding does; Keys & recovery's kit through `identity.kit_export`; Services through the engine's live status, as Home does.

### Changed from the design

- **Icons** are the library's set, not the design's: each of the design's icons has the nearest of the library's.
- **Tabs** are the library's pill tabs, whose chosen tab is dark; the design's is light grey. The chosen tab's count does not show inside the pill.
- **Control heights** are the library's (32 for a button and a field; the design draws 34).
- **The recovery kit's dialog** shows the places of the words, numbered, not sample words; the design's line says 12 words and the engine's kit has 24.
- **Rows that name another assistant are left out**: Import's three "Import from another AI app" rows and its "Start import" dialog, one of General's five places to open a file, and one row of Usage & billing's table. This repository names no other assistant.
- **Dialog choices** have the one option the design shows; the design draws no menu for them.

### For Omar to review: pages that may not belong

Taken over from other apps' settings, with nothing in Isonomic behind them today:

- **Parental controls**, **Trusted contact**: family and account-recovery features of a hosted account; Isonomic's account is a key on this Mac.
- **Mini & Pets**: a desktop companion.
- **Usage & billing** (plans, credits, "instant reset"), and **Account**'s plan row: a hosted subscription. Local models have no limits to reset.
- **Import**: brings setup from other assistants; its rows are left out above.
- **Appshots**, **Computer use** (and its browser extension), **Browser**, **Passwords**, **Cloud computer**, **Cloud environments**: a built-in browser, screen control and hosted machines that Isonomic does not have.
- **Code Review**, **Git**, **Worktrees**, **Hooks**, **Configuration** (`config.toml`, "Reinstall workspace"): a coding agent's settings.
- **Voice**, **Archived chats**, **Personalization**, **Data controls**: these belong to Chat more than to the window; where they live is a decision.
- **Account** and **Profile** say "passkey", "Secure Enclave", "smart account" and show an email and a handle: not true of the engine's ID today (the same words onboarding still carries).
- **Connections** ("Control this Mac", SSH), **File viewers**, **Cells**, **Backups**, **Storage**, **Notifications**, **Appearance**, **Keyboard shortcuts**, **General**: fit Isonomic; each needs its engine or window work.

### Checked

- `tests/settings.rs`: the files hold together (every page the sidebar names, every dialog a button opens), every page draws in both themes, every dialog opens and closes, a button opens what the design gives it and a row leads to its sub-page, what is set is kept and read back, the search finds pages and settings.
- Pictures of every page and dialog: `SETTINGS_SHOTS=<folder> cargo test -p shell --test settings_shots` (`SETTINGS_SHOTS_ONLY` narrows them), set beside the design's frames on the Intel Mac.

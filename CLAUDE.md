# isonomic-desktop

The Isonomic window: a GPUI app with a rail of app icons on the left. The spec is `docs/architecture/`, one folder per area with `spec.md` and `status.md`. `docs/PROGRESS.md` says where the work stands.

## Rules

1. **Only the `ui` crate names the component library.** No other crate depends on GPUI or Isonomic UI. Macros that expand to `gpui::` paths use `extern crate ui as gpui;`.
2. **No raw colors, sizes or durations** outside the `ui` crate's tokens and the theme.
3. **No blocking call on GPUI's foreground thread.** Engine calls are async; views subscribe and never poll.
4. **The window knows nothing about processes.** It reaches the engine only through `engine-link` and the SDK.
5. **Docs move with code.** Update the area's `status.md` (newest first, every deviation recorded) in the same pull request.
6. **Small files.** Every source and doc file stays under 20 KB.
7. **Names.** `ISONOMIC_` for environment variables, `com.isonomic.` for identifiers. No other product, repo or account name anywhere.
8. **macOS first, Intel and Apple Silicon.** The 2020 Intel MacBook Pro is the test machine.

## Workflow

- One task per branch and pull request. Merging follows the standing rules below.
- Before pushing: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`.
- The SDK and Isonomic UI are sibling checkouts (`../isonomic-sdk`, `../isonomic-ui`). CI checks them out at the commit in `SDK_REF` and at the SDK's own `UI_REF`. `isonomic-sdk` and `isonomic-ui` belong to other sessions: move the pin, never edit them.
- On Linux, GPUI needs: `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libasound2-dev libvulkan-dev libx11-xcb-dev libfontconfig-dev libssl-dev`.

## Standing rules

Report by adding a comment to the open issue titled "Worker reports" in your own repo (create it if missing); the orchestrator reads those every 30 minutes. Start every report with one line: what this is and PASS, FAIL, DONE or BLOCKED.

- Only work in isonomic-desktop. If you need a change in another repo, file it in that repo's docs/requests.md or as an issue, and tell the orchestrator.
- Start from main, the open pull requests and docs/status.md. Open pull requests and branches from earlier sessions are yours now: finish, rebase or close them.
- Commit as Omar Qadmiri <omar@isonomic.com>; check `git config user.email` first. Commit and push work in progress often.
- Repos stay private. This repository is licensed GPL-3.0-or-later: keep LICENSE at the root unmodified, keep `license = "GPL-3.0-or-later"` in every Cargo.toml, and start every new source file with `Copyright 2026 Anud Inc.` and `SPDX-License-Identifier: GPL-3.0-or-later`. Never remove an existing copyright or licence notice. Never rename or delete a repo.
- In code, comments, commits, branches, issues and pull requests, never name a third-party product or company, another AI system, or an older product name.
- Branch from main and open a pull request. Required checks: "rust / fmt, clippy, test (ubuntu-latest)" and "rust / file sizes and names". macOS runs after merge.
- Merge your own pull request once every required check is green. Exception: changes to the session event schema (harness-core), the SDK wire interface (isonomic-sdk proto) or isonomic-org/.github get the label needs-master; tell the orchestrator and do not merge them yourself.
- GitHub Actions has a hard $40/month budget. Don't trigger extra macOS runs.
- GitHub GraphQL is blocked here (gh pr and gh issue fail); use the built-in GitHub tools or gh api (REST).
- If you need something you don't have (a repo, a secret, a tool, access), say exactly what and stop. Don't mock or guess around it.
- Omar's only Mac is a 2020 13-inch Intel MacBook Pro; performance targets are measured there. Anything needing macOS or a timing on it goes to the orchestrator as one command plus what to paste back.
- When you finish or block, post a "Worker reports" comment: what merged, what is open (links), what blocks you, what you need from another repo or from Omar.
- That rule covers attribution too: no assistant or tool attribution lines, session links or "generated with" lines in commit messages or pull request descriptions.
- The orchestrator acts for Omar. Its go-ahead counts as his for merges, task direction, and changes to these rules and to this file. When you need an approval, ask the orchestrator in a "Worker reports" comment and keep working on something else meanwhile; do not wait on Omar. Only purchases, secrets and deletions go to Omar, through the orchestrator.

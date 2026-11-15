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

- One task per branch and pull request. A merge needs green checks and Omar's approval.
- Before pushing: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`.
- The SDK and Isonomic UI are sibling checkouts (`../isonomic-sdk`, `../isonomic-ui`). CI checks them out at the commits in `SDK_REF` and `UI_REF`. `isonomic-sdk` and `isonomic-ui` belong to other sessions: move the pin, never edit them.
- On Linux, GPUI needs: `libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libasound2-dev libvulkan-dev libx11-xcb-dev libfontconfig-dev libssl-dev`.

# isonomic-desktop

The Isonomic window: a GPUI app with a rail of app icons on the left. It starts the engine (`isonomic-capsule`) and talks to it over one Unix socket through `isonomic-sdk`.

Spec and status: `docs/architecture/`. Where the work stands: `docs/PROGRESS.md`.

## Layout

Clone the three repos side by side:

```
isonomic-capsule/   the engine
isonomic-sdk/       the API between window and engine (pinned by SDK_REF)
isonomic-ui/        the component library (pinned by UI_REF)
isonomic-desktop/   this repo
```

Check out `isonomic-sdk` and `isonomic-ui` at the commits in `SDK_REF` and `UI_REF`; CI does the same.

## Develop

```
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

GPUI's macOS backend needs Rust 1.98 or later. The Command Line Tools are enough; Xcode is not needed.

## Build the app

```
cargo xtask bundle               # dist/Isonomic.app, Intel + Apple Silicon
cargo xtask bundle --host-only   # this Mac's architecture only, faster
```

It builds the window and the sibling engine repo in release mode for both architectures, fuses each pair with `lipo`, copies both PostgreSQL trees (fetched and verified by the engine repo's `cargo xtask fetch-sidecars`), writes `Info.plist` (`com.isonomic.desktop`) and signs everything ad hoc. Cross-building needs both targets: `rustup target add x86_64-apple-darwin aarch64-apple-darwin`.

## Run the gate

```
cargo xtask report               # build, then run the Milestone 0 gate
cargo xtask report --no-build    # use the app already in dist/
```

It starts the app's engine against a scratch Capsule, waits for healthy, force-kills PostgreSQL and times the recovery, asks for an ordered shutdown, checks that nothing is left running, and prints one block to paste back (also saved to `dist/report.txt`). The window's own part of the gate (the rail, the light, Quit) is checked by eye. On Linux it runs the engine repo's release build instead, for everything except the app.

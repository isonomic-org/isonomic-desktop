# Packaging: status

The engine side of packaging (install shapes, bundled services) is recorded in `isonomic-capsule/docs/status.md`. This area's history up to this repo's commit `eb30c136` moved to `isonomic-capsule/docs/status/packaging.md`.

Newest first. Changes to this repo's `cargo xtask bundle` and the app bundle go below.

## 2026-11-15 · `--example-app` for a check bundle

- `cargo xtask bundle --example-app` builds the window with the `example-app` feature, so the bundle mounts the SDK's Hello app. It's for checking app hosting and deep links by eye, never for a release. The report always builds without it.
- The bundle's `Info.plist` registers the `isonomic` URL scheme (desktop #28).

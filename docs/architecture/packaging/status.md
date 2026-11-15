# Packaging: status

The engine side of packaging (install shapes, bundled services) is recorded in `isonomic-capsule/docs/status.md`. This area's history up to this repo's commit `eb30c136` moved to `isonomic-capsule/docs/status/packaging.md`.

Newest first. Changes to this repo's `cargo xtask bundle` and the app bundle go below.

## 2026-11-15 · `--example-app` for a check bundle

- `cargo xtask bundle --example-app` builds the window with the `example-app` feature, so the bundle mounts the SDK's Hello app. It's for checking app hosting and deep links by eye, never for a release. The report always builds without it.
- The bundle's `Info.plist` registers the `isonomic` URL scheme (desktop #28).

## 2026-11-15 · Release details short of signing

- **Universal:** `cargo xtask bundle` without `--host-only` builds both slices and fuses them (unchanged).
- **Version and build:** `CFBundleShortVersionString` is the workspace version. `CFBundleVersion` is the build number: `ISONOMIC_BUILD_NUMBER` when set, else the branch's commit count. The bundle prints both.
- **Icon slot:** `packaging/macos/AppIcon-1024.png`. When it's there, the bundle makes `Resources/AppIcon.icns` (all ten iconset sizes, with `sips` and `iconutil`) and sets `CFBundleIconFile`. When it isn't, the bundle prints that the app has no icon and carries on. No icon is approved yet.
- **Signing dry run:** `--sign-dry-run` looks for a Developer ID Application certificate (`security find-identity`; `ISONOMIC_SIGN_IDENTITY` picks one) and prints the codesign commands a release would run: hardened runtime, secure timestamp, nested code first. Without a certificate it prints "signing: skipped". It never signs; the app stays ad-hoc signed.
- **The model server's layout:** the engine looks for `llama-server` in `Resources/sidecars/llama-server-<triple>/` and also accepts a fused `Resources/sidecars/llama-server/` (isonomic-capsule `packaging::Bundle::binary`). The bundle ships whatever the engine's `sidecars.json` declares, so the model server lands there once the engine un-defers it. Nothing changes here.
- **Not checked on a Mac:** the icon and the dry run run only on macOS; their logic is unit-tested on Linux.

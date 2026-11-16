# Packaging: status

The engine side of packaging (install shapes, bundled services) is recorded in `isonomic-capsule/docs/status.md`. This area's history up to this repo's commit `eb30c136` moved to `isonomic-capsule/docs/status/packaging.md`.

Newest first. Changes to this repo's `cargo xtask bundle` and the app bundle go below.

## 2026-11-16 · The bundle carries the acceptance check

- `Contents/Resources/acceptance-check` (mode 755), from `packaging/macos/acceptance-check.sh`, copied before signing, so the signature seals it. See verification's status.

## 2026-11-16 · The login agent's standard error is `logs/engine.stderr`

- **What changed:** the engine writes its own rotating `logs/engine.log` (isonomic-capsule #71). The plist's `StandardErrorPath` now points at `logs/engine.stderr`, so the two never write the same file.
- **Existing installs:** the agent is rewritten at the next launch, since its content changed.

## 2026-11-16 · Where problem reports go is set at release

- **Compiled in:** `ISONOMIC_REPORTS_URL` and `ISONOMIC_REPORTS_KEY` in the environment of `cargo xtask bundle` or `release` go into the window when it is compiled. The bundle prints "problem reports: sent to <host>, with a key" or "not sent (no ISONOMIC_REPORTS_URL); kept on the Mac". It never prints the key.
- **The key can be read:** it is compiled into the app, so anyone with the app can extract it. It should only allow posting reports.

## 2026-11-16 · The engine's reports carry the build

- **What's set:** when the bundle compiles the engine, it sets `ISONOMIC_BUILD` (the bundle's build number, the same as `CFBundleVersion` and the window's) and `ISONOMIC_ENGINE_COMMIT` (the engine checkout's full commit). The engine reads both at compile time for its problem reports.
- **Refused:** a bundle whose engine checkout isn't a git checkout.

## 2026-11-16 · The app ships the licence notices

- **Shipped:** `cargo xtask bundle` copies the engine's `sidecars/NOTICES.txt` to `Contents/Resources/NOTICES.txt`. The engine now builds the database and the cache from pinned source, so we distribute our own builds and their licences travel with the app.
- **Refused:** the bundle, and therefore `cargo xtask release`, fails when the engine's file is missing, or when the app has no `NOTICES.txt` or an empty one.
- **Shown:** the window's Isonomic menu has Acknowledgements (see the window status).
- **Tested:** shipping the file, and refusing an app without it or with an empty one.

## 2026-11-16 · The minimum-macOS refusal names each part

- **Why:** build 106's release was refused, correctly, because the bundled database declares macOS 26.0 on both architectures and the cache 15.0. The cause took reading a list of files to see.
- **Now:** the refusal starts with one line per part and architecture: the part's name (a sidecar without its target triple, or the program), the highest minimum among its files, and how many files are above 13.3. The first ten files follow, each with its architecture, then a count of the rest.
- **How:** `macho::slices` reads each slice's architecture with its minimum, so a fused file reports both. `macho::summarize` groups the files.
- **Not changed:** the product's minimum stays 13.3. Fixing the sidecars is the engine worker's; raising the minimum is Omar's decision.

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

## 2026-11-15 · The app icon

- The artwork is `assets/icon/app-icon-1024.png` (desktop #36, from Omar's design file). It fills its canvas edge to edge, so the bundle places it on the macOS icon grid: scaled to 824 pixels with Lanczos3 and centred on a transparent 1024 canvas. It then writes all ten iconset sizes itself, so only `iconutil` runs on the Mac to make `Resources/AppIcon.icns`.
- The empty slot at `packaging/macos/AppIcon-1024.png` is gone.
- Tested on Linux with the supplied artwork: the grid margin is transparent, the middle is the artwork, and every iconset size has the right dimensions.

## 2026-11-15 · The unsigned release folder: `cargo xtask release`

Version 1 ships without a Developer ID (no account for at least two weeks), as a download.

- **`cargo xtask release`** (macOS) builds the universal, ad-hoc signed app and writes `dist/release/`, which is what gets handed over:
  - `Isonomic-<version>-<build>.dmg`: the app, a link to Applications, and the first-open note; the app icon is on the volume.
  - `….dmg.sha256` in `shasum -a 256` format.
  - `….json`: version, build, file, size, hash, minimum macOS, architectures, `signed: ad-hoc`, `notarized: false`.
  - `Opening Isonomic the first time.md`: the same note, beside the download.
  
  Nothing here builds, links to or knows about a website.
- **First open:** `packaging/macos/first-open.md` covers moving the app to Applications, the Gatekeeper dialog, **Open Anyway** in Privacy & Security, and checking the hash. Its key steps are unit-tested to stay in the note.
- **Signing later is a switch, not new work:**
  - `packaging/macos/entitlements.plist` holds the hardened runtime's one exception: library validation off, for the bundled services' own libraries.
  - The signing dry run passes the entitlements, and a notarization dry run prints `notarytool submit`, `stapler staple` and `spctl`.
  - Both are skipped with one line when the keychain has no Developer ID certificate.

**What assumes a signed build, checked:**

| Part | Unsigned (ad hoc) |
| --- | --- |
| The engine's token key | A file in the Capsule (`secrets/token.key`), not the keychain: unaffected |
| Keychain access groups | None used anywhere in the window or engine |
| The `isonomic` URL scheme | Registered from `Info.plist` when the app is first opened or moved; signing isn't required |
| Start at login (launch agent) | Works, but only once the app is in Applications: see translocation below |
| App Translocation | **Degrades.** A quarantined app opened from the disk image or Downloads runs from a random read-only path that changes every launch. The window now detects it (`engine_link::translocated`), installs no login agent and asks the person on Home to move the app |
| Permissions macOS remembers per app (microphone and the like) | None used yet. An ad-hoc signature changes with every build, so any such grant would be asked again after each update |

**To check on the Mac:** a real first open from the disk image (Gatekeeper's dialog, Open Anyway, then the engine started by the launch agent at the next login).

## 2026-11-16 · Minimum macOS is 13.3, and the bundle checks it

- **The first universal release built on the Intel Mac:** `Isonomic-0.0.1-96.dmg`, 125.7 MB, both architectures, ad-hoc signed. Signing and notarization were skipped cleanly. The acceptance check passed: prompt, stop and continue, and the link.
- **The manifest's 11.0 was wrong.** Read from the binaries: 39 of the model server's 40 Intel files and 41 of its 42 Apple Silicon files name macOS **13.3** as their minimum; one helper names 11.0. The app now declares 13.3 (`LSMinimumSystemVersion`, the manifest and the first-open note).
- **The bundle refuses to drift:**
  - Our own programs are built with `MACOSX_DEPLOYMENT_TARGET` at the declared minimum.
  - After assembling, the bundle reads every Mach-O file's minimum from its load commands (`xtask/src/macho.rs`: thin and universal files, `LC_BUILD_VERSION` and `LC_VERSION_MIN_MACOSX`).
  - It fails, naming the files, if any part needs more than the app declares.
- **Not read here:** the database and cache sidecars for macOS. The cache is built from source and only builds on a Mac. The check covers both on the next Mac build.

**Version numbering, proposed for V1:**
- `CFBundleShortVersionString`, the workspace version, the manifest's `version` and the file name: `0.9.N` for builds people outside test (0.9.0 first), then `1.0.0` for V1. Fixes after that are `1.0.1` and so on. It's always three numbers, as macOS requires.
- `CFBundleVersion`, the build: the commit count on `main`, 96 today. It only goes up, so any two builds compare correctly even across versions; `ISONOMIC_BUILD_NUMBER` can override it.
- The file name stays `Isonomic-<version>-<build>.dmg`.
- Nothing changes until it's approved: the workspace version is still 0.0.1.

## 2026-11-16 · Version 0.9.0

- The numbering proposed above is approved (orchestrator, for Omar): `0.9.N` for test builds starting at 0.9.0, then `1.0.0` for version 1, and the build number from the commit count on `main`. The workspace version is now 0.9.0, so the next release is `Isonomic-0.9.0-<build>.dmg`.

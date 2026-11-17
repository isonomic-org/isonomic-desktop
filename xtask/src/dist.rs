// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! `cargo xtask release`: the release folder, handed to whoever publishes
//! the download.
//!
//! ```text
//! dist/release/
//!   Isonomic-<version>-<build>.dmg          the app, an Applications link and the first-open note
//!   Isonomic-<version>-<build>.dmg.sha256   `shasum -a 256` format
//!   Isonomic-<version>-<build>.json         version, build, file, size, hash, minimum macOS, architectures
//!   Opening Isonomic the first time.md      the same note, beside the download
//! ```
//!
//! The app is universal and ad-hoc signed (`bundle`). Developer ID signing,
//! notarization and stapling run as a dry run: skipped with no certificate,
//! printed with one. Nothing here links to or knows about a website.

use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::bundle::{self, APP_NAME, MINIMUM_MACOS};
use crate::{release, util};

/// The note a person reads before opening the app the first time.
pub const FIRST_OPEN: &str = "packaging/macos/first-open.md";
pub const FIRST_OPEN_NAME: &str = "Opening Isonomic the first time.md";

pub fn run() -> Result<(), String> {
    let built = bundle::run(&bundle::Options {
        host_only: false,
        example_app: false,
        sign_dry_run: true,
        // A release is never the check build.
        check_build: false,
    })?;
    let desktop = util::desktop_root();
    let version = env!("CARGO_PKG_VERSION");
    let build = release::build_number(&desktop);
    let out = desktop.join("dist/release");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;

    let stem = file_stem(version, &build);
    let dmg = out.join(format!("{stem}.dmg"));
    disk_image(&built.app, &desktop.join(FIRST_OPEN), version, &dmg)?;
    let bytes = std::fs::read(&dmg).map_err(|e| format!("reading {}: {e}", dmg.display()))?;
    let hash = sha256_hex(&bytes);
    let name = format!("{stem}.dmg");
    std::fs::write(
        out.join(format!("{name}.sha256")),
        format!("{hash}  {name}\n"),
    )
    .map_err(|e| format!("writing the hash: {e}"))?;
    let arches: Vec<&str> = built
        .targets
        .iter()
        .map(|t| t.split('-').next().unwrap_or(t))
        .collect();
    let manifest = manifest(version, &build, &name, bytes.len() as u64, &hash, &arches);
    std::fs::write(out.join(format!("{stem}.json")), manifest)
        .map_err(|e| format!("writing the manifest: {e}"))?;
    std::fs::copy(desktop.join(FIRST_OPEN), out.join(FIRST_OPEN_NAME))
        .map_err(|e| format!("copying the first-open note: {e}"))?;
    release::notarize_dry_run(&dmg);
    println!("release folder: {}", out.display());
    for entry in std::fs::read_dir(&out)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        println!(
            "  {:<48} {:>9}",
            entry.file_name().to_string_lossy(),
            util::megabytes(size)
        );
    }
    Ok(())
}

/// `Isonomic-<version>-<build>`: stable, sortable, and carries the version.
pub fn file_stem(version: &str, build: &str) -> String {
    format!("{APP_NAME}-{version}-{build}")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn manifest(
    version: &str,
    build: &str,
    file: &str,
    size: u64,
    sha256: &str,
    architectures: &[&str],
) -> String {
    let value = serde_json::json!({
        "name": APP_NAME,
        "version": version,
        "build": build,
        "file": file,
        "size_bytes": size,
        "sha256": sha256,
        "minimum_macos": MINIMUM_MACOS,
        "architectures": architectures,
        "signed": "ad-hoc",
        "notarized": false,
    });
    serde_json::to_string_pretty(&value).expect("a JSON value") + "\n"
}

/// A compressed disk image holding the app, a link to Applications and the
/// first-open note, with the app icon on the volume when the tools exist.
fn disk_image(app: &Path, note: &Path, version: &str, dmg: &Path) -> Result<(), String> {
    let stage: PathBuf = std::env::temp_dir().join(format!("isonomic-dmg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| format!("staging: {e}"))?;
    util::run_quiet(
        Command::new("ditto")
            .arg(app)
            .arg(stage.join(format!("{APP_NAME}.app"))),
        "copying the app",
    )?;
    std::os::unix::fs::symlink("/Applications", stage.join("Applications"))
        .map_err(|e| format!("linking Applications: {e}"))?;
    std::fs::copy(note, stage.join(FIRST_OPEN_NAME)).map_err(|e| format!("the note: {e}"))?;
    let icns = app.join(format!("Contents/Resources/{}.icns", release::ICON_NAME));
    if icns.is_file() {
        let _ = std::fs::copy(&icns, stage.join(".VolumeIcon.icns"));
    }
    let made = util::run(
        Command::new("hdiutil")
            .args(["create", "-quiet", "-ov", "-format", "UDZO", "-fs", "HFS+"])
            .arg("-volname")
            .arg(format!("{APP_NAME} {version}"))
            .arg("-srcfolder")
            .arg(&stage)
            .arg(dmg),
        "making the disk image",
    );
    let _ = std::fs::remove_dir_all(&stage);
    made
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_carries_the_version_and_build() {
        assert_eq!(file_stem("0.0.1", "214"), "Isonomic-0.0.1-214");
    }

    #[test]
    fn the_hash_is_sha256_in_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_manifest_says_what_the_download_is() {
        let text = manifest(
            "0.0.1",
            "214",
            "Isonomic-0.0.1-214.dmg",
            1234,
            "ab",
            &["x86_64", "aarch64"],
        );
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["version"], "0.0.1");
        assert_eq!(value["build"], "214");
        assert_eq!(value["size_bytes"], 1234);
        assert_eq!(value["minimum_macos"], MINIMUM_MACOS);
        assert_eq!(value["architectures"][1], "aarch64");
        assert_eq!(value["notarized"], false);
    }

    #[test]
    fn the_first_open_note_names_the_steps() {
        let note = std::fs::read_to_string(util::desktop_root().join(FIRST_OPEN)).unwrap();
        for step in [
            "Applications",
            "Privacy & Security",
            "Open Anyway",
            "shasum -a 256",
        ] {
            assert!(note.contains(step), "{step}");
        }
    }
}

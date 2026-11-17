// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Release details for the bundle: the build number, the app icon, and a
//! dry run of Developer ID signing.
//!
//! - **Build number** (`CFBundleVersion`): `ISONOMIC_BUILD_NUMBER` when set
//!   (a release sets it), else the number of commits on this branch, so
//!   every build from a newer commit counts up.
//! - **Icon:** the artwork at [`ICON_SOURCE`], a 1024 by 1024 PNG that fills
//!   its canvas edge to edge. It is placed on the macOS icon grid (scaled to
//!   [`GRID_ART`] pixels, centred on a transparent 1024 canvas), resized to
//!   every iconset size here, and turned into `Resources/AppIcon.icns` by
//!   the system's `iconutil`. With no artwork the app has no icon and the
//!   bundle says so; nothing fails.
//! - **Signing dry run** (`--sign-dry-run`): look for a Developer ID
//!   Application certificate in the keychain and print the commands a
//!   signed release would run. With no certificate it prints that signing
//!   is skipped. It never signs: the bundle stays ad-hoc signed.

use std::path::Path;
use std::process::Command;

use crate::util;

/// Where the app icon comes from, relative to the repo root.
pub const ICON_SOURCE: &str = "assets/icon/app-icon-1024.png";
/// The macOS icon canvas, in pixels.
pub const CANVAS: u32 = 1024;
/// How much of the canvas the artwork covers on the macOS icon grid: 824
/// of 1024 pixels, leaving the system's margin for the shadow.
pub const GRID_ART: u32 = 824;
/// The icon's file name inside `Contents/Resources`, without `.icns`.
pub const ICON_NAME: &str = "AppIcon";
/// Overrides the build number.
pub const ENV_BUILD_NUMBER: &str = "ISONOMIC_BUILD_NUMBER";
/// Picks one Developer ID identity when the keychain holds several.
pub const ENV_SIGN_IDENTITY: &str = "ISONOMIC_SIGN_IDENTITY";
/// The `notarytool` keychain profile a signed release submits with.
pub const ENV_NOTARY_PROFILE: &str = "ISONOMIC_NOTARY_PROFILE";
/// The entitlements a Developer ID signature carries, relative to the repo.
pub const ENTITLEMENTS: &str = "packaging/macos/entitlements.plist";

/// The build number: the override, else this branch's commit count.
pub fn build_number(repo: &Path) -> String {
    if let Ok(number) = std::env::var(ENV_BUILD_NUMBER) {
        if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return number;
        }
    }
    util::output("git", &["rev-list", "--count", "HEAD"], Some(repo)).unwrap_or_else(|| "0".into())
}

/// The sizes an `.iconset` holds: (pixels, file name).
pub fn iconset_sizes() -> Vec<(u32, String)> {
    let mut sizes = Vec::new();
    for points in [16u32, 32, 128, 256, 512] {
        sizes.push((points, format!("icon_{points}x{points}.png")));
        sizes.push((points * 2, format!("icon_{points}x{points}@2x.png")));
    }
    sizes
}

/// The artwork on the macOS icon grid: scaled to [`GRID_ART`] and centred
/// on a transparent [`CANVAS`].
pub fn on_grid(art: &image::RgbaImage) -> image::RgbaImage {
    use image::imageops::{overlay, resize, FilterType};
    let scaled = resize(art, GRID_ART, GRID_ART, FilterType::Lanczos3);
    let mut canvas = image::RgbaImage::new(CANVAS, CANVAS);
    let at = i64::from((CANVAS - GRID_ART) / 2);
    overlay(&mut canvas, &scaled, at, at);
    canvas
}

/// Write every iconset size of `icon` (a [`CANVAS`]-sized image) into `set`.
pub fn write_iconset(icon: &image::RgbaImage, set: &Path) -> Result<(), String> {
    use image::imageops::{resize, FilterType};
    std::fs::create_dir_all(set).map_err(|e| format!("creating {}: {e}", set.display()))?;
    for (pixels, name) in iconset_sizes() {
        let sized = if pixels == CANVAS {
            icon.clone()
        } else {
            resize(icon, pixels, pixels, FilterType::Lanczos3)
        };
        sized
            .save(set.join(&name))
            .map_err(|e| format!("writing {name}: {e}"))?;
    }
    Ok(())
}

/// Make `Resources/AppIcon.icns` from the artwork. `Ok(false)` when there
/// is no artwork.
pub fn icon(repo: &Path, resources: &Path) -> Result<bool, String> {
    let source = repo.join(ICON_SOURCE);
    if !source.is_file() {
        println!("icon: none at {ICON_SOURCE}; the app has no icon");
        return Ok(false);
    }
    let art = image::open(&source)
        .map_err(|e| format!("reading {ICON_SOURCE}: {e}"))?
        .to_rgba8();
    if art.width() != art.height() {
        return Err(format!(
            "{ICON_SOURCE} is {}x{}, not square",
            art.width(),
            art.height()
        ));
    }
    let set = std::env::temp_dir().join(format!("isonomic-{}.iconset", std::process::id()));
    let _ = std::fs::remove_dir_all(&set);
    write_iconset(&on_grid(&art), &set)?;
    std::fs::create_dir_all(resources).map_err(|e| format!("creating Resources: {e}"))?;
    let icns = resources.join(format!("{ICON_NAME}.icns"));
    let made = util::run_quiet(
        Command::new("iconutil")
            .args(["--convert", "icns", "--output"])
            .arg(&icns)
            .arg(&set),
        "making AppIcon.icns",
    );
    let _ = std::fs::remove_dir_all(&set);
    made.map(|()| true)
}

/// The Developer ID identities `security` lists, one per line as it prints
/// them: `1) <hash> "Developer ID Application: …"`.
pub fn developer_ids(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| {
            let start = line.find('"')? + 1;
            let end = line[start..].find('"')? + start;
            Some(line[start..end].to_string())
        })
        .filter(|name| name.starts_with("Developer ID Application:"))
        .collect()
}

/// The Developer ID identity a signed release would use, if the keychain
/// has one.
pub fn identity() -> Option<String> {
    let listing = util::output(
        "security",
        &["find-identity", "-v", "-p", "codesigning"],
        None,
    )
    .unwrap_or_default();
    let ids = developer_ids(&listing);
    match std::env::var(ENV_SIGN_IDENTITY).ok() {
        Some(name) => ids.into_iter().find(|id| id.contains(name.as_str())),
        None => ids.into_iter().next(),
    }
}

/// Print the notarization a signed release would run for `dmg`, or that it
/// is skipped.
pub fn notarize_dry_run(dmg: &Path) {
    if identity().is_none() {
        println!("notarization: skipped (no Developer ID Application certificate)");
        return;
    }
    let profile = std::env::var(ENV_NOTARY_PROFILE).unwrap_or_else(|_| "<keychain profile>".into());
    println!("notarization (dry run, nothing submitted):");
    for line in notarize_commands(dmg, &profile) {
        println!("  {line}");
    }
}

pub fn notarize_commands(dmg: &Path, profile: &str) -> Vec<String> {
    let dmg = dmg.display();
    vec![
        format!("codesign --force --timestamp --sign \"$IDENTITY\" \"{dmg}\""),
        format!("xcrun notarytool submit \"{dmg}\" --keychain-profile \"{profile}\" --wait"),
        format!("xcrun stapler staple \"{dmg}\""),
        format!("spctl --assess --type open --context context:primary-signature -v \"{dmg}\""),
    ]
}

/// Print what a signed release would run, or that signing is skipped.
pub fn sign_dry_run(app: &Path) {
    let Some(identity) = identity() else {
        println!("signing: skipped (no Developer ID Application certificate in the keychain)");
        return;
    };
    println!("signing (dry run, nothing signed) with \"{identity}\":");
    for line in sign_commands(app, &identity) {
        println!("  {line}");
    }
}

/// The commands, nested code first and the app last, with the hardened
/// runtime and a secure timestamp, as notarization requires.
pub fn sign_commands(app: &Path, identity: &str) -> Vec<String> {
    let app = app.display();
    vec![
        format!(
            "find \"{app}/Contents/Resources/sidecars\" -type f -perm +111 -exec codesign --force --options runtime --timestamp --sign \"{identity}\" {{}} +"
        ),
        format!("codesign --force --options runtime --timestamp --entitlements {ENTITLEMENTS} --sign \"{identity}\" \"{app}/Contents/MacOS/isonomic-capsule\""),
        format!("codesign --force --options runtime --timestamp --entitlements {ENTITLEMENTS} --sign \"{identity}\" \"{app}\""),
        format!("codesign --verify --strict --deep \"{app}\""),
    ]
}

/// Say where the window's problem reports go: the URL is compiled in from
/// `ISONOMIC_REPORTS_URL`, the key from `ISONOMIC_REPORTS_KEY`, which is
/// never printed.
pub fn report_destination() {
    let url = std::env::var("ISONOMIC_REPORTS_URL").unwrap_or_default();
    let key = std::env::var("ISONOMIC_REPORTS_KEY").is_ok_and(|k| !k.is_empty());
    if url.is_empty() {
        println!("problem reports: not sent (no ISONOMIC_REPORTS_URL); kept on the Mac");
    } else {
        let host = url.split_once("://").map_or(&*url, |(_, r)| r);
        let host = host.split(['/', '?', '#']).next().unwrap_or(host);
        let key = if key { "with a key" } else { "without a key" };
        println!("problem reports: sent to {host}, {key}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_iconset_has_ten_sizes_up_to_1024() {
        let sizes = iconset_sizes();
        assert_eq!(sizes.len(), 10);
        assert_eq!(
            sizes.last().unwrap(),
            &(1024, "icon_512x512@2x.png".to_string())
        );
    }

    #[test]
    fn the_supplied_artwork_lands_on_the_macos_grid() {
        let source = util::desktop_root().join(ICON_SOURCE);
        let art = image::open(&source).unwrap().to_rgba8();
        assert_eq!((art.width(), art.height()), (CANVAS, CANVAS));
        let icon = on_grid(&art);
        assert_eq!((icon.width(), icon.height()), (CANVAS, CANVAS));
        // The margin is transparent; the middle is the artwork.
        let margin = (CANVAS - GRID_ART) / 2;
        assert_eq!(icon.get_pixel(margin / 2, CANVAS / 2)[3], 0);
        assert_eq!(icon.get_pixel(CANVAS / 2, margin / 2)[3], 0);
        assert!(icon.get_pixel(CANVAS / 2, CANVAS / 2)[3] > 200);

        let set = tempfile_dir();
        write_iconset(&icon, &set).unwrap();
        for (pixels, name) in iconset_sizes() {
            let written = image::open(set.join(&name)).unwrap();
            assert_eq!(
                (written.width(), written.height()),
                (pixels, pixels),
                "{name}"
            );
        }
        let _ = std::fs::remove_dir_all(&set);
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("iconset-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn only_developer_id_application_identities_count() {
        let listing = r#"  1) 0123 "Apple Development: Someone (ABC)"
  2) 4567 "Developer ID Application: Isonomic (TEAM123)"
     2 valid identities found"#;
        assert_eq!(
            developer_ids(listing),
            ["Developer ID Application: Isonomic (TEAM123)"]
        );
        assert!(developer_ids("     0 valid identities found").is_empty());
    }

    #[test]
    fn signing_commands_end_with_the_app_and_a_verify() {
        let commands = sign_commands(
            Path::new("/x/Isonomic.app"),
            "Developer ID Application: I (T)",
        );
        assert!(commands[2].ends_with("\"/x/Isonomic.app\""));
        assert!(
            commands.iter().all(|c| !c.contains("--sign -")),
            "not ad hoc"
        );
        assert!(commands[3].starts_with("codesign --verify"));
        assert!(commands[2].contains(ENTITLEMENTS), "{}", commands[2]);
        assert!(util::desktop_root().join(ENTITLEMENTS).is_file());
        let notarize = notarize_commands(Path::new("/x/I.dmg"), "p");
        assert!(notarize[1].starts_with("xcrun notarytool submit"));
        assert!(notarize[2].starts_with("xcrun stapler staple"));
    }

    #[test]
    fn the_build_number_counts_commits_unless_overridden() {
        let number = build_number(&util::desktop_root());
        assert!(
            number.chars().all(|c| c.is_ascii_digit() || c == '.'),
            "{number}"
        );
    }
}

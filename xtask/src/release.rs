//! Release details for the bundle: the build number, the app icon, and a
//! dry run of Developer ID signing.
//!
//! - **Build number** (`CFBundleVersion`): `ISONOMIC_BUILD_NUMBER` when set
//!   (a release sets it), else the number of commits on this branch, so
//!   every build from a newer commit counts up.
//! - **Icon:** a 1024 by 1024 PNG at [`ICON_SOURCE`]. When it is there, the
//!   bundle gets `Resources/AppIcon.icns`, made with the system's `sips`
//!   and `iconutil`. When it is not, the app has no icon and the bundle
//!   says so; nothing fails.
//! - **Signing dry run** (`--sign-dry-run`): look for a Developer ID
//!   Application certificate in the keychain and print the commands a
//!   signed release would run. With no certificate it prints that signing
//!   is skipped. It never signs: the bundle stays ad-hoc signed.

use std::path::Path;
use std::process::Command;

use crate::util;

/// Where the app icon comes from, relative to the repo root.
pub const ICON_SOURCE: &str = "packaging/macos/AppIcon-1024.png";
/// The icon's file name inside `Contents/Resources`, without `.icns`.
pub const ICON_NAME: &str = "AppIcon";
/// Overrides the build number.
pub const ENV_BUILD_NUMBER: &str = "ISONOMIC_BUILD_NUMBER";
/// Picks one Developer ID identity when the keychain holds several.
pub const ENV_SIGN_IDENTITY: &str = "ISONOMIC_SIGN_IDENTITY";

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

/// Make `Resources/AppIcon.icns` from the source PNG. `Ok(false)` when
/// there is no source.
pub fn icon(repo: &Path, resources: &Path) -> Result<bool, String> {
    let source = repo.join(ICON_SOURCE);
    if !source.is_file() {
        println!("icon: none at {ICON_SOURCE}; the app has no icon");
        return Ok(false);
    }
    let set = std::env::temp_dir().join(format!("isonomic-{}.iconset", std::process::id()));
    let _ = std::fs::remove_dir_all(&set);
    std::fs::create_dir_all(&set).map_err(|e| format!("creating {}: {e}", set.display()))?;
    for (pixels, name) in iconset_sizes() {
        let size = pixels.to_string();
        util::run_quiet(
            Command::new("sips")
                .args(["-z", &size, &size])
                .arg(&source)
                .arg("--out")
                .arg(set.join(name)),
            "resizing the app icon",
        )?;
    }
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

/// Print what a signed release would run, or that signing is skipped.
pub fn sign_dry_run(app: &Path) {
    let listing = util::output(
        "security",
        &["find-identity", "-v", "-p", "codesigning"],
        None,
    )
    .unwrap_or_default();
    let ids = developer_ids(&listing);
    let wanted = std::env::var(ENV_SIGN_IDENTITY).ok();
    let chosen = match &wanted {
        Some(name) => ids.iter().find(|id| id.contains(name.as_str())).cloned(),
        None => ids.first().cloned(),
    };
    let Some(identity) = chosen else {
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
        format!("codesign --force --options runtime --timestamp --sign \"{identity}\" \"{app}/Contents/MacOS/isonomic-capsule\""),
        format!("codesign --force --options runtime --timestamp --sign \"{identity}\" \"{app}\""),
        format!("codesign --verify --strict --deep \"{app}\""),
    ]
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

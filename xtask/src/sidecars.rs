// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The engine's bundled services, as the app ships them.
//!
//! The engine repo's `sidecars.json` says which services are bundled (not
//! deferred, not draft) and their install shape; its `cargo xtask
//! fetch-sidecars` has already verified each one into `sidecars/`.
//!
//! - A **tree** (`install: directory`) ships once per architecture as
//!   `<name>-<triple>/`, minus headers, readmes, stamps and build logs; the
//!   running engine picks its own slice's tree.
//! - A **single program** (file or build shape) is fused for every target
//!   into `<name>/<program>`, which the engine finds when it has no
//!   per-architecture tree.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util;

/// What a tree install leaves out of the app.
const TREE_SKIP: [&str; 3] = ["include", ".sha256", ".cache"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidecar {
    pub name: String,
    pub tree: bool,
}

/// The services the app bundles, from the engine repo's manifest.
pub fn bundled(capsule: &Path) -> Result<Vec<Sidecar>, String> {
    let path = capsule.join("sidecars.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn parse(text: &str) -> Result<Vec<Sidecar>, String> {
    let manifest: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let entries = manifest["sidecars"].as_array().ok_or("no sidecars list")?;
    let flag = |entry: &serde_json::Value, key: &str| entry[key].as_bool() == Some(true);
    Ok(entries
        .iter()
        .filter(|entry| !flag(entry, "deferred") && !flag(entry, "draft"))
        .filter_map(|entry| {
            Some(Sidecar {
                name: entry["name"].as_str()?.to_string(),
                tree: entry["install"].as_str() == Some("directory"),
            })
        })
        .collect())
}

/// Copy or fuse every bundled service into `dest` (`Contents/Resources/sidecars`).
pub fn ship(capsule: &Path, targets: &[String], dest: &Path) -> Result<(), String> {
    let installed = capsule.join("sidecars");
    for sidecar in bundled(capsule)? {
        let sources: Vec<PathBuf> = targets
            .iter()
            .map(|t| installed.join(format!("{}-{t}", sidecar.name)))
            .collect();
        for source in &sources {
            if !source.join(".sha256").is_file() {
                return Err(format!(
                    "{} is missing: run `cargo xtask fetch-sidecars` in the engine repo",
                    source.display()
                ));
            }
        }
        if sidecar.tree {
            for source in &sources {
                copy_tree(source, &dest.join(source.file_name().expect("named")))?;
            }
        } else {
            let program = program_in(&sources[0])?;
            let slices: Vec<PathBuf> = sources.iter().map(|s| s.join(&program)).collect();
            let out = dest.join(&sidecar.name);
            std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
            crate::bundle::fuse(&slices, &out.join(&program))?;
        }
    }
    Ok(())
}

/// The licence notices the engine writes for what it bundles, shipped as
/// `Contents/Resources/NOTICES.txt` and shown by Acknowledgements.
pub const NOTICES: &str = "NOTICES.txt";

/// Copy the engine's `sidecars/NOTICES.txt` into `resources`.
pub fn ship_notices(capsule: &Path, resources: &Path) -> Result<(), String> {
    let source = capsule.join("sidecars").join(NOTICES);
    if !source.is_file() {
        return Err(format!(
            "{} is missing: run `cargo xtask fetch-sidecars` in the engine repo",
            source.display()
        ));
    }
    std::fs::copy(&source, resources.join(NOTICES))
        .map(|_| ())
        .map_err(|e| format!("copying {}: {e}", source.display()))
}

/// Refuse an app whose notices are missing or empty: we distribute our
/// own builds of what it bundles, so their licences must travel with it.
pub fn check_notices(app: &Path) -> Result<(), String> {
    let path = app.join("Contents/Resources").join(NOTICES);
    match std::fs::metadata(&path) {
        Ok(meta) if meta.len() > 0 => {
            println!(
                "notices: Contents/Resources/{NOTICES}, {} bytes",
                meta.len()
            );
            Ok(())
        }
        _ => Err(format!(
            "the app has no licence notices at Contents/Resources/{NOTICES}"
        )),
    }
}

fn skipped(name: &str) -> bool {
    TREE_SKIP.contains(&name) || name.starts_with("README") || name.ends_with(".log")
}

fn copy_tree(source: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let entries = std::fs::read_dir(source).map_err(|e| format!("{}: {e}", source.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if skipped(&name) {
            continue;
        }
        // ditto keeps symlinks (shared libraries use them) and modes.
        util::run(
            Command::new("ditto").arg(entry.path()).arg(to.join(&name)),
            &format!("copying {}", entry.path().display()),
        )?;
    }
    Ok(())
}

/// The one program in a single-program install: not hidden, not a log.
fn program_in(dir: &Path) -> Result<String, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| !n.starts_with('.') && !n.ends_with(".log"))
        .collect();
    match names.len() {
        1 => Ok(names.remove(0)),
        _ => Err(format!(
            "{}: expected one program, found {names:?}",
            dir.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notices_are_shipped_and_an_app_without_them_is_refused() {
        let root = std::env::temp_dir().join(format!("notices-test-{}", std::process::id()));
        let (capsule, app) = (root.join("capsule"), root.join("Isonomic.app"));
        let resources = app.join("Contents/Resources");
        std::fs::create_dir_all(capsule.join("sidecars")).unwrap();
        std::fs::create_dir_all(&resources).unwrap();
        assert!(ship_notices(&capsule, &resources).is_err());
        assert!(check_notices(&app).is_err());
        std::fs::write(
            capsule.join("sidecars").join(NOTICES),
            "database: licence text",
        )
        .unwrap();
        ship_notices(&capsule, &resources).unwrap();
        check_notices(&app).unwrap();
        std::fs::write(resources.join(NOTICES), "").unwrap();
        assert!(check_notices(&app).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn only_bundled_services_with_their_shape() {
        let manifest = r#"{ "sidecars": [
            { "name": "postgres", "install": "directory" },
            { "name": "valkey", "build": {} },
            { "name": "seaweedfs", "deferred": true },
            { "name": "next", "draft": true } ] }"#;
        let found = parse(manifest).unwrap();
        assert_eq!(
            found,
            [
                Sidecar {
                    name: "postgres".into(),
                    tree: true
                },
                Sidecar {
                    name: "valkey".into(),
                    tree: false
                },
            ]
        );
    }

    #[test]
    fn a_tree_drops_headers_readmes_stamps_and_logs() {
        for name in ["include", ".sha256", "README.md", "build.log"] {
            assert!(skipped(name), "{name}");
        }
        for name in ["bin", "lib", "share", "LICENSE", "COPYRIGHT"] {
            assert!(!skipped(name), "{name}");
        }
    }

    #[test]
    fn a_single_program_install_names_its_program() {
        let dir = std::env::temp_dir().join(format!("xtask-sidecar-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".sha256"), "x").unwrap();
        std::fs::write(dir.join("cache-server"), "x").unwrap();
        assert_eq!(program_in(&dir).unwrap(), "cache-server");
        std::fs::write(dir.join("other"), "x").unwrap();
        assert!(program_in(&dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

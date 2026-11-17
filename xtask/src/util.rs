// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small helpers shared by the tasks: where the repos are, running commands,
//! measuring sizes.

use std::path::{Path, PathBuf};
use std::process::Command;

/// This repo's root.
pub fn desktop_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask sits in the repo root")
        .to_path_buf()
}

/// A repo cloned next to this one.
pub fn sibling(name: &str) -> PathBuf {
    desktop_root()
        .parent()
        .expect("the repo has a parent directory")
        .join(name)
}

/// The engine repo, which must be cloned next to this one.
pub fn capsule_root() -> Result<PathBuf, String> {
    let root = sibling("isonomic-capsule");
    if root.join("Cargo.toml").is_file() {
        Ok(root)
    } else {
        Err(format!(
            "the engine repo is missing: clone isonomic-capsule at {}",
            root.display()
        ))
    }
}

/// The cargo that runs this task, so child builds use the same toolchain.
pub fn cargo(dir: &Path) -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(dir);
    command
}

/// Where cargo puts a repo's build output: `CARGO_TARGET_DIR` when it is set
/// (both repos then share it, which cargo supports), else `<repo>/target`.
pub fn target_dir(repo: &Path) -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) if !dir.is_empty() => repo.join(dir),
        _ => repo.join("target"),
    }
}

/// Run a command to completion; fail with `what` if it does not succeed.
pub fn run(command: &mut Command, what: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("{what}: cannot start: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{what} failed ({status})"))
    }
}

/// Like [`run`], but keeps the command's output unless it fails.
pub fn run_quiet(command: &mut Command, what: &str) -> Result<(), String> {
    let out = command
        .output()
        .map_err(|error| format!("{what}: cannot start: {error}"))?;
    if out.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let tail: Vec<&str> = stderr.lines().rev().take(10).collect();
    let tail: Vec<&str> = tail.into_iter().rev().collect();
    Err(format!(
        "{what} failed ({}):\n{}",
        out.status,
        tail.join("\n")
    ))
}

/// Trimmed standard output of a command, if it ran and succeeded.
pub fn output(program: &str, args: &[&str], dir: Option<&Path>) -> Option<String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let out = command.output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Bytes on disk under `path`, not following symlinks.
pub fn dir_size(path: &Path) -> u64 {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !meta.is_dir() {
        return meta.len();
    }
    std::fs::read_dir(path)
        .map(|entries| entries.flatten().map(|e| dir_size(&e.path())).sum())
        .unwrap_or(0)
}

/// Bytes as megabytes with one decimal (1 MB = 1,000,000 bytes).
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}

/// Short commit hash of a repo, or `?`.
pub fn git_rev(dir: &Path) -> String {
    let rev = output("git", &["rev-parse", "--short", "HEAD"], Some(dir));
    let dirty = output("git", &["status", "--porcelain"], Some(dir)).is_some_and(|s| !s.is_empty());
    match rev {
        Some(rev) if dirty => format!("{rev} (uncommitted changes)"),
        Some(rev) => rev,
        None => "?".to_string(),
    }
}

/// The last `count` lines of a text file.
pub fn tail(path: &Path, count: usize) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let skip = lines.len().saturating_sub(count);
    lines.into_iter().skip(skip).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_tails() {
        let dir = std::env::temp_dir().join(format!("xtask-util-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a")).unwrap();
        std::fs::write(dir.join("a/one"), [0u8; 300]).unwrap();
        std::fs::write(dir.join("two"), "x\ny\nz\n").unwrap();
        assert_eq!(dir_size(&dir), 306);
        assert_eq!(tail(&dir.join("two"), 2), ["y", "z"]);
        assert_eq!(megabytes(72_900_000), "72.9 MB");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

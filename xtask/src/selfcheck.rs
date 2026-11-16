//! `cargo xtask self-check --models <dir> [--capsule <dir>] [--prompt <text>]`:
//! the acceptance check for a release build, on a Mac.
//!
//! Runs the window inside `dist/Isonomic.app` twice, as its own instance
//! with a fresh scratch Capsule, never through `open` or the URL scheme:
//! first `ISONOMIC_SELF_CHECK=<prompt>` (first run, Chat, a turn, then stop
//! and continue), then `ISONOMIC_SELF_CHECK=link` with `isonomic://chat/home`
//! as a launch argument. The model files in `--models` are copied into the
//! scratch Capsule first. Prints each run's `[self-check]` lines and fails
//! unless both pass.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{bundle, util};

const DEFAULT_PROMPT: &str = "Plan one day in Lisbon";
const LINK: &str = "isonomic://chat/home";

pub struct Options {
    pub models: PathBuf,
    pub capsule: PathBuf,
    pub prompt: String,
}

impl Options {
    /// From `--models`, `--capsule` and `--prompt`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let value = |name: &str| {
            args.iter()
                .position(|arg| arg == name)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };
        let models =
            value("--models").ok_or("--models <dir> is required: the model files to install")?;
        Ok(Self {
            models: models.into(),
            capsule: value("--capsule")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("isonomic-self-check")),
            prompt: value("--prompt").unwrap_or_else(|| DEFAULT_PROMPT.into()),
        })
    }
}

pub fn run(options: &Options) -> Result<(), String> {
    let app = bundle::app_path();
    let window = app.join("Contents/MacOS").join(bundle::WINDOW_BIN);
    if !window.is_file() {
        return Err(format!(
            "no app at {}: run `cargo xtask bundle` first",
            app.display()
        ));
    }
    fresh_capsule(&options.capsule, &options.models)?;
    let prompt = check(&window, &options.capsule, &options.prompt, None)?;
    let link = check(&window, &options.capsule, "link", Some(LINK))?;
    match (prompt, link) {
        (true, true) => {
            println!("self-check: PASS (prompt and link)");
            Ok(())
        }
        (prompt, link) => Err(format!(
            "self-check: FAIL (prompt {}, link {})",
            word(prompt),
            word(link)
        )),
    }
}

fn word(passed: bool) -> &'static str {
    if passed {
        "passed"
    } else {
        "failed"
    }
}

fn fresh_capsule(capsule: &Path, models: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(capsule);
    let into = capsule.join("models");
    std::fs::create_dir_all(&into).map_err(|e| format!("creating {}: {e}", into.display()))?;
    let entries =
        std::fs::read_dir(models).map_err(|e| format!("reading {}: {e}", models.display()))?;
    for entry in entries.flatten() {
        let from = entry.path();
        if from.is_file() {
            let to = into.join(entry.file_name());
            std::fs::copy(&from, &to).map_err(|e| format!("copying {}: {e}", from.display()))?;
        }
    }
    Ok(())
}

/// One run of the window in check mode; `true` when it exited 0.
fn check(window: &Path, capsule: &Path, mode: &str, link: Option<&str>) -> Result<bool, String> {
    let log = capsule.join(format!(
        "self-check-{}.log",
        if link.is_some() { "link" } else { "prompt" }
    ));
    let file =
        std::fs::File::create(&log).map_err(|e| format!("creating {}: {e}", log.display()))?;
    let mut command = Command::new(window);
    command
        .env("ISONOMIC_CAPSULE_DIR", capsule)
        .env("ISONOMIC_SELF_CHECK", mode)
        .stdout(std::process::Stdio::null())
        .stderr(file);
    if let Some(link) = link {
        command.arg(link);
    }
    let status = command
        .status()
        .map_err(|e| format!("starting {}: {e}", window.display()))?;
    for line in util::tail(&log, 400) {
        if line.contains("[self-check]") {
            println!("{line}");
        }
    }
    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_are_required_and_the_rest_has_defaults() {
        assert!(Options::parse(&[]).is_err());
        let args: Vec<String> = ["--models", "/m"].iter().map(|s| s.to_string()).collect();
        let options = Options::parse(&args).unwrap();
        assert_eq!(options.models, PathBuf::from("/m"));
        assert_eq!(options.prompt, DEFAULT_PROMPT);
    }

    #[test]
    fn a_fresh_capsule_holds_only_the_models() {
        let dir = std::env::temp_dir().join(format!("self-check-test-{}", std::process::id()));
        let models = dir.join("models-src");
        std::fs::create_dir_all(&models).unwrap();
        std::fs::write(models.join("m.gguf"), b"weights").unwrap();
        let capsule = dir.join("Capsule");
        std::fs::create_dir_all(capsule.join("settings")).unwrap();
        std::fs::write(capsule.join("settings/setup-done"), b"").unwrap();
        fresh_capsule(&capsule, &models).unwrap();
        assert!(capsule.join("models/m.gguf").is_file());
        assert!(
            !capsule.join("settings/setup-done").exists(),
            "a fresh first run"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

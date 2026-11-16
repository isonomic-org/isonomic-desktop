//! `cargo xtask self-check --models <dir> [--capsule <dir>] [--prompt <text>]`:
//! the acceptance check for a release build, on a Mac.
//!
//! Runs the window inside `dist/Isonomic.app` twice, as its own instance
//! with a fresh scratch Capsule, never through `open` or the URL scheme:
//! first `ISONOMIC_SELF_CHECK=<prompt>` (first run, Chat, a turn, then stop
//! and continue), then `ISONOMIC_SELF_CHECK=link` with `isonomic://chat/home`
//! as a launch argument. Each run gets its own fresh scratch Capsule, with
//! the model files in `--models` linked or copied in: a scratch Capsule's
//! ID keys live only as long as its engine (below), so a second engine on
//! the same Capsule could not sign for an app. Prints each run's
//! `[self-check]` lines, and a failed run's last engine and window log
//! lines; fails unless both pass.
//!
//! The scratch Capsule is marked `"scratch": true` in its `capsule.json`
//! before the engine first opens it, and every run sets
//! `ISONOMIC_KEY_STORE=memory`: the engine then keeps the Isonomic ID's keys
//! in memory, and writes nothing to the person's keychain. A run passes
//! only if the engine's log confirms it ([`MEMORY_NOTE`]).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{bundle, util};

const DEFAULT_PROMPT: &str = "Plan one day in Lisbon";
const LINK: &str = "isonomic://chat/home";
/// The engine's line when it keeps the ID's keys in memory (isonomic-capsule
/// `identity::store_choice::MEMORY_NOTE`).
pub const MEMORY_NOTE: &str = "keys kept in memory: scratch Capsule";
/// The engine's own variable for its key store.
const ENV_KEY_STORE: &str = "ISONOMIC_KEY_STORE";

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
    let prompt = prompt && keys_in_memory(&options.capsule, 1);
    // A fresh Capsule again: the first engine took its keys with it.
    fresh_capsule(&options.capsule, &options.models)?;
    let link = check(&window, &options.capsule, "link", Some(LINK))?;
    let link = link && keys_in_memory(&options.capsule, 1);
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

/// How many of each log's last lines a failed run prints.
const LAST_LINES: usize = 20;

/// After a failed run: the engine's and the window's last log lines, so a
/// failure on a Mac explains itself. Home paths print as `~`.
fn print_last_lines(capsule: &Path) {
    let home = std::env::var("HOME").ok().filter(|h| h.len() > 1);
    for name in ["engine.stderr", "engine.log", "window.log"] {
        let path = capsule.join("logs").join(name);
        let lines = util::tail(&path, LAST_LINES);
        if lines.is_empty() {
            continue;
        }
        println!("--- last {} lines of logs/{name}", lines.len());
        for line in lines {
            match &home {
                Some(home) => println!("{}", line.replace(home.as_str(), "~")),
                None => println!("{line}"),
            }
        }
    }
}

/// Did the engine confirm, in its log, that its `runs` runs on this
/// Capsule each kept the ID's keys in memory? Prints the answer.
fn keys_in_memory(capsule: &Path, runs: usize) -> bool {
    let log = std::fs::read_to_string(capsule.join("logs").join("engine.log")).unwrap_or_default();
    let (confirmed, refusal) = key_store_lines(&log);
    if confirmed >= runs {
        println!("keys: memory (scratch Capsule), nothing written to the keychain");
        return true;
    }
    println!(
        "keys: FAIL, the engine did not confirm a memory key store{}",
        refusal.map(|r| format!(": {r}")).unwrap_or_default()
    );
    false
}

/// How many times the engine's log says it kept keys in memory, and its
/// last line about the key store if it refused.
fn key_store_lines(log: &str) -> (usize, Option<String>) {
    let confirmed = log.lines().filter(|l| l.contains(MEMORY_NOTE)).count();
    let refusal = log
        .lines()
        .rev()
        .find(|l| l.contains(ENV_KEY_STORE))
        .map(|l| l.trim().to_string());
    (confirmed, refusal)
}

fn fresh_capsule(capsule: &Path, models: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(capsule);
    let into = capsule.join("models");
    std::fs::create_dir_all(&into).map_err(|e| format!("creating {}: {e}", into.display()))?;
    // Marked before the engine first opens it: only a scratch Capsule may
    // keep its keys in memory, and the engine keeps the mark.
    let marker = capsule.join("capsule.json");
    std::fs::write(&marker, "{\n  \"format\": 1,\n  \"scratch\": true\n}\n")
        .map_err(|e| format!("writing {}: {e}", marker.display()))?;
    let entries =
        std::fs::read_dir(models).map_err(|e| format!("reading {}: {e}", models.display()))?;
    for entry in entries.flatten() {
        let from = entry.path();
        if from.is_file() {
            let to = into.join(entry.file_name());
            // A link where the disk allows it: a model is gigabytes.
            if std::fs::hard_link(&from, &to).is_err() {
                std::fs::copy(&from, &to)
                    .map_err(|e| format!("copying {}: {e}", from.display()))?;
            }
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
        .env(ENV_KEY_STORE, "memory")
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
    if !status.success() {
        print_last_lines(capsule);
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
        let marker = std::fs::read_to_string(capsule.join("capsule.json")).unwrap();
        let marker: serde_json::Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(marker["scratch"], true, "marked before the engine opens it");
        assert_eq!(marker["format"], 1);
        assert!(
            !capsule.join("settings/setup-done").exists(),
            "a fresh first run"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_engine_s_log_confirms_memory_or_says_why_not() {
        let both = format!("start\n{MEMORY_NOTE}\nstop\nstart\n{MEMORY_NOTE}\n");
        assert_eq!(key_store_lines(&both), (2, None));
        let refused = "ISONOMIC_KEY_STORE=memory refused: this Capsule is not a scratch Capsule";
        assert_eq!(key_store_lines(refused), (0, Some(refused.to_string())));
        assert_eq!(key_store_lines(""), (0, None));
    }
}

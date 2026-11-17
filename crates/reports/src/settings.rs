// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The "Problem reports" setting: Ask each time (the default), Send
//! automatically, or Never send. Kept in the Capsule's
//! `settings/reports.json`. Where reports go is not a setting: it is the
//! build's configuration ([`crate::send::Destination`]). With none, nothing
//! is sent, whatever the setting says.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Ask,
    Always,
    Never,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Ask => "ask",
            Mode::Always => "always",
            Mode::Never => "never",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        match text {
            "ask" => Some(Mode::Ask),
            "always" => Some(Mode::Always),
            "never" => Some(Mode::Never),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReportSettings {
    pub mode: Mode,
}

fn file(capsule: &Path) -> PathBuf {
    capsule.join("settings").join("reports.json")
}

/// The setting, or the defaults when there is none or it doesn't read.
pub fn load(capsule: &Path) -> ReportSettings {
    let Some(value) = std::fs::read_to_string(file(capsule))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return ReportSettings::default();
    };
    ReportSettings {
        mode: value["mode"]
            .as_str()
            .and_then(Mode::parse)
            .unwrap_or_default(),
    }
}

pub fn save(capsule: &Path, settings: &ReportSettings) -> std::io::Result<()> {
    let path = file(capsule);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let value = json!({ "mode": settings.mode.as_str() });
    std::fs::write(
        path,
        serde_json::to_string_pretty(&value).map_err(std::io::Error::other)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_ask_with_nowhere_to_send_and_a_choice_is_kept() {
        let capsule = tempfile::tempdir().unwrap();
        assert_eq!(load(capsule.path()), ReportSettings::default());
        assert_eq!(load(capsule.path()).mode, Mode::Ask);
        let never = ReportSettings { mode: Mode::Never };
        save(capsule.path(), &never).unwrap();
        assert_eq!(load(capsule.path()), never);
        std::fs::write(file(capsule.path()), "not json").unwrap();
        assert_eq!(load(capsule.path()), ReportSettings::default());
    }
}

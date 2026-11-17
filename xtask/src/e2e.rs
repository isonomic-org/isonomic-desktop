// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! `cargo xtask e2e`: the window's engine link against the real engine.
//!
//! Builds the engine from `../isonomic-capsule`, fetches its bundled
//! services for this machine (the engine's own `fetch-sidecars`), then runs
//! `crates/engine-link/tests/real_engine.rs` with the engine and the
//! services directory named in `ISONOMIC_E2E_ENGINE` and
//! `ISONOMIC_E2E_BUNDLE`. Each run uses a fresh scratch Capsule.
//!
//! The database refuses to run as root, so the task does too.

use std::process::Command;

use crate::util;

const ENGINE_BIN: &str = "isonomic-capsule";

pub fn run() -> Result<(), String> {
    if running_as_root() {
        return Err(
            "run as an ordinary user: the engine's database refuses to start as root".into(),
        );
    }
    let capsule = util::capsule_root()?;
    util::run(
        util::cargo(&capsule).args(["build", "--package", ENGINE_BIN, "--bin", ENGINE_BIN]),
        "building the engine",
    )?;
    util::run(
        util::cargo(&capsule).args(["xtask", "fetch-sidecars"]),
        "fetching the engine's services",
    )?;
    let engine = util::target_dir(&capsule).join("debug").join(ENGINE_BIN);
    let bundle = capsule.join("sidecars");
    util::run(
        util::cargo(&util::desktop_root())
            .args([
                "test",
                "--package",
                "engine-link",
                "--test",
                "real_engine",
                "--",
                "--nocapture",
            ])
            .env("ISONOMIC_E2E_ENGINE", &engine)
            .env("ISONOMIC_E2E_BUNDLE", &bundle),
        "the end-to-end test",
    )
}

fn running_as_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .is_some_and(|out| String::from_utf8_lossy(&out.stdout).trim() == "0")
}

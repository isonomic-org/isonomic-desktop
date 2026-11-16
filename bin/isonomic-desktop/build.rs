//! Embed what a problem report says about this build: its number and the
//! SDK, Chat and engine commits it carries.

use std::path::Path;
use std::process::Command;

fn pin(file: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    std::fs::read_to_string(path)
        .map(|text| text.trim().chars().take(12).collect())
        .unwrap_or_else(|_| "unknown".into())
}

fn engine() -> String {
    let capsule = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../isonomic-capsule");
    Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(capsule)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn main() {
    println!("cargo:rerun-if-env-changed=ISONOMIC_BUILD_NUMBER");
    let build = std::env::var("ISONOMIC_BUILD_NUMBER").unwrap_or_else(|_| "dev".into());
    println!("cargo:rustc-env=ISONOMIC_REPORT_BUILD={build}");
    println!("cargo:rustc-env=ISONOMIC_REPORT_SDK={}", pin("SDK_REF"));
    println!("cargo:rustc-env=ISONOMIC_REPORT_CHAT={}", pin("CHAT_REF"));
    println!("cargo:rustc-env=ISONOMIC_REPORT_ENGINE={}", engine());
}

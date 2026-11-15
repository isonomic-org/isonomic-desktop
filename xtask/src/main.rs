//! Build tasks for the window repo.
//!
//! ```text
//! cargo xtask bundle [--host-only] [--example-app] [--sign-dry-run]
//!     Build dist/Isonomic.app: the window, the engine and PostgreSQL, as one
//!     universal (Intel + Apple Silicon) app, ad-hoc signed. macOS only.
//!
//! cargo xtask report [--host-only] [--no-build]
//!     Build, start the engine against a scratch Capsule, run the Milestone 0
//!     gate, and print one block to paste back.
//!
//! cargo xtask e2e
//!     Build the engine, fetch its services, and run the window's engine
//!     link against it in a scratch Capsule. Not as root.
//! ```
//!
//! The engine repo must be cloned next to this one (`../isonomic-capsule`),
//! like the SDK (`../isonomic-sdk`).

mod bundle;
mod e2e;
mod gate;
mod procs;
mod release;
mod report;
mod sidecars;
mod util;

use std::process::ExitCode;

const USAGE: &str = "usage:
  cargo xtask bundle [--host-only] [--example-app] [--sign-dry-run]
  cargo xtask report [--host-only] [--no-build]
  cargo xtask e2e";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|arg| arg == name);
    let result = match args.first().map(String::as_str) {
        Some("bundle") => bundle::run(&bundle::Options {
            host_only: flag("--host-only"),
            example_app: flag("--example-app"),
            sign_dry_run: flag("--sign-dry-run"),
        })
        .map(|built| built.print()),
        Some("e2e") => e2e::run(),
        Some("report") => report::run(&report::Options {
            host_only: flag("--host-only"),
            build: !flag("--no-build"),
        }),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

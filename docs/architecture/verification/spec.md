# Verification

Spec section 12.

- **Engine:** hermetic unit and integration tests with stand-in binaries, fake protocol servers and injected clocks. Runs anywhere, including an agent's sandbox.
- **Window:** headless UI tests for every view; screenshots written to `target/screenshots/` for the macOS CI job to upload.
- **One command.** `cargo xtask report` builds, starts the engine against a scratch Capsule, runs the milestone's gate, and prints a block the owner can paste back: versions, where each binary resolved, cold-start time, gate result, last log lines on failure.
- **Docs move with code.** Each area has `spec.md` and `status.md`. Status is newest first and records every deviation.
- **Before pushing:** `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.

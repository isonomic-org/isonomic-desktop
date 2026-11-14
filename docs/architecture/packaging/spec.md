# Packaging

Spec section 11.

- **One universal `.app`** containing the window, the engine and the services. Single-file binaries are fused for both architectures; directory trees ship once per architecture and the running slice picks its own.
- **Sidecar manifest.** Pinned URLs and SHA-256 hashes, three install shapes (file, directory, build from source), draft entries, an architecture check on every artifact including cache hits. Fail closed. Binaries are never committed. The fetch step is a Rust task; the repo needs no other toolchain.
- **Startup report.** The engine logs one line per bundled binary: where it resolved and whether it is executable.
- **Signing.** Ad-hoc during development. Developer ID and notarization at the end.
- **Size.** Bundle only the PostgreSQL tools the engine actually runs.

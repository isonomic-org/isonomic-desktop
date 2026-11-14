# Lessons from the earlier build

Spec section 1. Treat every line as a rule.

## Engineering

| Lesson | Rule in this design |
| --- | --- |
| The core never depended on the shell. It took binary paths as plain inputs | Keep that seam. The engine knows nothing about windows; the window knows nothing about processes |
| Whoever asks for runtime state must own the desired state | The engine holds desired state for everything it manages and reconciles after any restart |
| A readiness probe that speaks the real protocol beats a port check, and cannot be misdirected by the user's environment | Every service has a protocol-level probe. No probe shells out to a client tool |
| The user's environment leaks into child processes | The engine builds each child's environment from an allow-list. Nothing is inherited by default |
| Write the registry before notifying subscribers | Persist, then publish. Applies to every state change the window can observe |
| Never reuse software the user already has installed | Everything the engine runs ships inside the app, pinned and verified |
| Upstream releases are often a directory, not a file | Three install shapes (file, directory, build from source), a manifest with hashes, and an architecture check on every artifact, including cache hits |
| Draft entries let unproven assumptions land without breaking builds | Keep draft mode for any pinned artifact not yet confirmed against a real download |
| Packaging settings for one OS broke builds on another | Per-platform packaging config from day one |
| Laptop sleep and kill are the normal case | Every service stops with its own signal and deadline; PostgreSQL gets recovery time on start; every user edit is in the engine before the UI confirms it |
| A hung service looks alive | Two failed steady-state probes with the process alive means kill and restart |
| The honest lever on a desktop is not starting things | Heavy services start on first use; the RAM gate refuses rather than swaps |
| Tests that depend on the machine's free memory or real timers flake | Injectable clocks, hardware sources and stand-in binaries everywhere. Hermetic tests are the default |

## Process

| Lesson | Rule |
| --- | --- |
| The loop that caught real bugs: the agent writes and merges, the owner runs one command on his Mac and pastes the output | Every milestone ends with one command that prints a self-describing report. The app logs one line per service at startup saying where it resolved and how long it took |
| CI was the only compiler for a long time | CI must compile and test everything, including a macOS job. Anything verifiable off-Mac (the whole engine) is verified before pushing |
| Large files could not be written through the agent's write path | Keep source files and docs under 20 KB. Split by responsibility |
| Agents could not edit workflow files | CI lives in one shared workflow set up once; repos only call it |
| Spec and status drifted when kept in one file | Each area has a spec and a separate status file, newest first |
| Phase gates only work if they are hard | Each milestone has one gate test. No starting the next until it passes |

## Product

| Lesson | Rule |
| --- | --- |
| The biggest risk was signing per-app launchers generated at install time | Gone. Apps are tabs |
| Passkeys inside a web view were fragile | Replaced by a spike: native passkey calls with the key-wrapping extension |
| "Credential-scoped, not isolated" was the honest description of the app sandbox | Still honest for first-party apps. Real isolation exists only where it is mechanical: Views and Cells |
| The failure mode is shipping 60% of everything | One vertical slice at a time. The first milestone is a window showing a supervised database, nothing more |
| A web UI kit kept the design undecided | Isonomic UI is the design. The window never contains a raw color or size |

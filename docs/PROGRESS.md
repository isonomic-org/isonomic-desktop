# Progress

Newest first. Updated at every issue boundary so a new session can resume from this repo alone.

## 2026-11-14 · Milestone 0, Step 0 (push, build, see): blocked on repo bootstrap

No product code has been written. Step 0 of the Milestone 0 kickoff brief has to pass first, and one part of it fails.

| Check | Result | Evidence |
| --- | --- | --- |
| Repos exist | **Failed** | `isonomic-capsule`, `isonomic-sdk` and the org-level `.github` repo return 404. This repo holds only `README.md` and `.gitignore`: no `CLAUDE.md`, no Cargo workspace, no `ci.yml`. |
| Push | Works in this repo | A branch, this commit and a draft pull request were created through the GitHub connector. Not testable for the other two repos until they exist. |
| Engine builds off-Mac | Confirmed for the code to be ported | The earlier supervisor crate builds and passes all 66 of its tests (61 unit, 5 integration) on Linux x86_64 with Rust 1.97, in about one minute. The crates.io registry is reachable from the build sandbox. |
| Window builds on macOS CI | **Not possible yet** | There is no shared workflow and no GPUI hello-world to compile. |
| File size | Rule adopted | Every source and doc file stays under 20 KB. |
| CI files | Untested | Nothing to call until the shared workflow exists. |

### Needed from Omar

1. Run `bootstrap.sh` from the scaffold bundle. It creates the missing repos, pushes the scaffolds (including `CLAUDE.md` and `ci.yml` here) and turns on the shared CI.
2. Confirm the GitHub connector can see the new repos afterwards. If they still return 404, the connector's repository access for the org needs to include them.

### Notes for whoever resumes

- The build sandbox has no git access to this org. Reads and writes go through the GitHub connector, one file at a time, so the sandbox copy is the working tree and the connector push is the publish step.
- CI results are readable through the connector (check runs on a pull request), so a paste-back is only needed for what CI cannot show: the window itself and `cargo xtask report` on the Intel test machine.
- Component library pin candidate for the `ui` crate: `gpui-kit` 0.9.1, the latest release on crates.io on this date. To be pinned with `=` in the first window pull request.

### Next, once Step 0 passes

1. Read `README.md` and `CLAUDE.md` in all three repos, and the shared workflow.
2. Commit the architecture spec into `docs/architecture/` on this branch: one folder per area, each with `spec.md` and `status.md`.
3. Open the Milestone 0 issues as briefs, in dependency order: engine core, supervisor, registry, PostgreSQL, fetch task, broker, SDK, window, packaging, report.

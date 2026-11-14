# Window: status

Newest first.

## 2026-11-14 (later) · Skeleton built and tested on Linux; pull request #2

**Built**

- `ui`: the single re-export, pinned to `gpui-kit = "=0.9.1"`, plus the layout sizes the shell uses. No other crate names GPUI or GPUI Kit.
- `engine-link`: computes the socket path with the rule shared through the SDK, starts the engine binary when nothing is listening (at most once per 10 seconds), and owns one reconnecting client on its own single-thread Tokio runtime.
- `shell`: the window with the left rail, a tab host and the Home tab. The Home tab keeps the latest status and redraws when the link's watch changes. Nothing polls, and no engine call happens on GPUI's foreground thread.
- `bin/isonomic-desktop`: opens the window.

**Verified**

- GPUI Kit 0.9.1 compiles on Linux, so window code has a real compiler without a Mac. The workspace compiles, links, and passes fmt, clippy and 7 tests there.
- Two headless tests open the real shell in an off-screen window and walk it through: connecting, database healthy, restarting after a kill, healthy again, engine gone. This is the window half of gate items 1 and 2, without a screen.

**Deviations**

- GPUI's macros expand to `gpui::` paths, so a crate that uses them adds `extern crate ui as gpui;`. It is an alias for the `ui` crate, not a dependency on GPUI. Only the test file needs it today.
- The rail shows the tab's first letter in place of an icon until the icon set exists.
- GPUI has its own executor, not Tokio. The link therefore runs the connection on a Tokio runtime of its own, and views await only channel types that work on any executor.
- `Cargo.lock` is not committed: at 225 KB it is over the size the GitHub connector can write.

**Not done**

- Nothing has been built or run on macOS. The first run on the Intel test machine is still ahead, and still comes before any UI investment.
- Menu bar icon, hide on close, Quit asking the engine to shut down, single instance, the app bundle. All macOS-specific.
- Shell state stored in the engine; tabs other than Home.

## 2026-11-14 · Not started

- Component library pin candidate: `gpui-kit` 0.9.1.

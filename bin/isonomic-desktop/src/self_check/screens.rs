//! The full self-check's screenshots: the window's own rendering of its
//! last frame (`Window::render_to_image`), never a capture of the screen,
//! so nothing else on the Mac is ever in them. Saved as
//! `<Capsule>/screens/NN-name.png` at the window's size, light theme, and
//! for the main screens once more in dark (`NN-name-dark.png`).
//!
//! Only the check build (`--features check-build`) can render to an image;
//! in any other build every shot is skipped and says so once.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use ui::{AnyWindowHandle, App};

static DIR: OnceLock<PathBuf> = OnceLock::new();
static NEXT: AtomicUsize = AtomicUsize::new(1);

/// Save screenshots under `capsule/screens` from now on.
pub fn start(capsule: &Path) -> PathBuf {
    let dir = capsule.join("screens");
    let _ = std::fs::create_dir_all(&dir);
    let _ = DIR.set(dir.clone());
    dir
}

/// Whether this build can take them.
pub fn available() -> bool {
    cfg!(feature = "check-build")
}

/// The next file name for `name`, numbered in order.
fn next_path(name: &str, dark: bool) -> Option<PathBuf> {
    let dir = DIR.get()?;
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let suffix = if dark { "-dark" } else { "" };
    Some(dir.join(format!("{n:02}-{name}{suffix}.png")))
}

/// One light screenshot of the window as it is now.
pub fn shot(window: AnyWindowHandle, name: &str, cx: &mut App) -> Result<PathBuf, String> {
    capture(window, name, false, cx)
}

/// A light and a dark screenshot, for a main screen; the theme is put back.
pub fn both(window: AnyWindowHandle, name: &str, cx: &mut App) -> Result<PathBuf, String> {
    let light = capture(window, name, false, cx)?;
    let dark = themed(window, name, cx);
    dark.map(|_| light)
}

#[cfg(feature = "check-build")]
fn themed(window: AnyWindowHandle, name: &str, cx: &mut App) -> Result<PathBuf, String> {
    use ui::component::theme::{Theme, ThemeMode};
    let was = ui::component::theme::ActiveTheme::theme(cx).mode;
    Theme::change(ThemeMode::Dark, None, cx);
    let shot = capture(window, name, true, cx);
    Theme::change(was, None, cx);
    let _ = window.update(cx, |_, window, _| window.refresh());
    shot
}

#[cfg(not(feature = "check-build"))]
fn themed(_: AnyWindowHandle, _: &str, _: &mut App) -> Result<PathBuf, String> {
    Err(NOT_THIS_BUILD.into())
}

/// Why a build without the feature takes none.
pub const NOT_THIS_BUILD: &str =
    "no screenshot: this is not the check build (cargo xtask bundle --check-build)";

#[cfg(feature = "check-build")]
fn capture(
    window: AnyWindowHandle,
    name: &str,
    dark: bool,
    cx: &mut App,
) -> Result<PathBuf, String> {
    use ui::test::TestWindowExt as _;
    let path = next_path(name, dark).ok_or("screenshots were not started")?;
    let image = window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            window.render_to_image()
        })
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("rendering the window: {e}"))?;
    image
        .save(&path)
        .map_err(|e| format!("saving {}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(not(feature = "check-build"))]
fn capture(_: AnyWindowHandle, name: &str, dark: bool, _: &mut App) -> Result<PathBuf, String> {
    let _ = next_path(name, dark);
    Err(NOT_THIS_BUILD.into())
}

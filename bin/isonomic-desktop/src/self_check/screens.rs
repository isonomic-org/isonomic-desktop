// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

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

/// How many pixels in the middle of the element `id` differ from its
/// commonest colour: zero when nothing is drawn on its fill, such as an
/// icon that failed to load. Only the central half is counted, where an
/// icon sits, so a button's own fill and its rounded corners don't count.
/// The check build only.
#[cfg(feature = "check-build")]
pub fn ink_in(window: AnyWindowHandle, id: &'static str, cx: &mut App) -> Result<usize, String> {
    use std::collections::HashMap;
    use ui::test::TestWindowExt as _;
    window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            let bounds = window
                .try_find(id)
                .ok_or(format!("no {id} on screen"))?
                .bounds();
            let image = window
                .render_to_image()
                .map_err(|e| format!("rendering the window: {e}"))?;
            let scale = image.width() as f32 / f32::from(window.viewport_size().width);
            let at = |v: ui::Pixels| (f32::from(v) * scale) as u32;
            let (left, top) = (at(bounds.left()), at(bounds.top()));
            let (w, h) = (at(bounds.size.width), at(bounds.size.height));
            let (x0, y0) = (left + w / 4, top + h / 4);
            let (x1, y1) = (
                (left + w * 3 / 4).min(image.width()),
                (top + h * 3 / 4).min(image.height()),
            );
            if x0 >= x1 || y0 >= y1 {
                return Err(format!("{id} has no area"));
            }
            let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
            for y in y0..y1 {
                for x in x0..x1 {
                    *counts.entry(image.get_pixel(x, y).0).or_default() += 1;
                }
            }
            let fill = counts.values().copied().max().unwrap_or(0);
            let total: usize = counts.values().sum();
            Ok(total - fill)
        })
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use ui::assets::IconName;
    use ui::AssetSource as _;

    /// The asset source the window registers serves Isonomic UI's icons.
    #[test]
    fn the_window_s_asset_source_has_the_icons() {
        for icon in [IconName::Plus, IconName::Check, IconName::ArrowLeft] {
            let bytes = ui::assets::AllAssets.load(&icon.path()).unwrap();
            assert!(bytes.is_some_and(|b| !b.is_empty()), "{icon:?}");
        }
    }
}

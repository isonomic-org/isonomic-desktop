//! The single re-export of the component library.
//!
//! Every other crate in this repo depends on `ui` and nothing else for its
//! interface: no crate names GPUI or GPUI Kit directly, so swapping in
//! Isonomic UI, or upgrading the pinned version, is a change to this crate
//! alone.
//!
//! GPUI's macros expand to paths that start with `gpui::`. A crate that uses
//! them (`#[ui::test]`, derives) adds `extern crate ui as gpui;` at its root;
//! that is an alias for this crate, not a dependency on GPUI.

pub use gpui_kit::*;

/// Layout sizes used by the shell. No view contains a raw size or color:
/// colors come from the theme, sizes from here or from the spacing scale.
pub mod tokens {
    use gpui_kit::{px, size, Pixels, Size};

    /// Size of the main window when it first opens.
    pub fn window_size() -> Size<Pixels> {
        size(px(1100.), px(720.))
    }

    /// Width of the left rail of app icons.
    pub fn rail_width() -> Pixels {
        px(56.)
    }

    /// Diameter of a status dot.
    pub fn status_dot() -> Pixels {
        px(8.)
    }
}

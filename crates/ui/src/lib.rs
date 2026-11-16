//! The single re-export of the component library.
//!
//! The library is Isonomic UI, reached through the SDK (`isonomic_app::ui`)
//! at the commit in the SDK's `UI_REF`, the one pin. Every other crate in
//! this repo depends on `ui` and nothing else for its interface: no crate
//! names GPUI or Isonomic UI directly, so moving the pin is a change to
//! `SDK_REF` alone.
//!
//! GPUI's macros expand to paths that start with `gpui::`. A crate that uses
//! them (`#[ui::test]`, derives) adds `extern crate ui as gpui;` at its root;
//! that is an alias for this crate, not a dependency on GPUI.

pub use isonomic_app::ui::*;

/// Layout sizes used by the shell. No view contains a raw size or color:
/// colors come from the theme, sizes from here or from the spacing scale.
pub mod tokens {
    use isonomic_app::ui::{px, size, Pixels, Size};

    /// Size of the main window when it first opens.
    pub fn window_size() -> Size<Pixels> {
        size(px(1100.), px(720.))
    }

    /// The widest a page of text and controls grows, so lines stay
    /// readable on a wide window.
    pub fn content_width() -> Pixels {
        px(640.)
    }

    /// Height of the scrolling block that shows a problem report's bytes.
    pub fn report_bytes_height() -> Pixels {
        px(360.)
    }

    /// How long after an engine status change the window looks at the
    /// kept problem reports again: the engine writes a report after its
    /// own watch sees the failure, so the file can follow the status.
    pub fn report_rescan_delay() -> std::time::Duration {
        std::time::Duration::from_secs(3)
    }

    /// Width of the left rail of app icons.
    pub fn rail_width() -> Pixels {
        px(56.)
    }

    /// Diameter of a status dot.
    pub fn status_dot() -> Pixels {
        px(8.)
    }

    /// Colors of the menu bar status light, as RGBA bytes. The menu bar icon
    /// is drawn outside GPUI, so it cannot read the theme.
    pub mod light {
        pub const HEALTHY: [u8; 4] = [52, 199, 89, 255];
        pub const STARTING: [u8; 4] = [255, 179, 0, 255];
        pub const DEGRADED: [u8; 4] = [255, 69, 58, 255];
        pub const STOPPED: [u8; 4] = [142, 142, 147, 255];
        /// Side of the square icon, in pixels.
        pub const ICON_SIZE: u32 = 36;
    }
}

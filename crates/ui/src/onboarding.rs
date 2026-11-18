// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes, places, colours and timings of the onboarding window, from the
//! design ("Onboarding 0.4": "01 · App launch" and "02 · Get started"; the
//! third screen's are in `terms`).
//! Places are in the window's own 1280 × 800, from its top left.

use isonomic_app::ui::{point, px, rgb, rgba, size, BoxShadow, Hsla, Pixels, Point, Size};

/// The window: fixed, with rounded corners and no title bar.
pub fn window_size() -> Size<Pixels> {
    size(px(1280.), px(800.))
}
pub const WIDTH: f32 = 1280.;
pub const HEIGHT: f32 = 800.;
pub fn corner() -> Pixels {
    px(10.)
}

pub fn page() -> Hsla {
    rgb(0xffffff).into()
}
pub fn ink() -> Hsla {
    rgb(0x111827).into()
}
pub fn hairline() -> Hsla {
    rgb(0xe5e7eb).into()
}

/// The mark: 116 across in the centre for its intro, then 40 across with
/// its corner at (40, 32).
pub const MARK_INTRO: f32 = 116.;
pub const MARK_REST: f32 = 40.;
pub const MARK_REST_CENTRE: (f32, f32) = (60., 52.);
/// On its way to the corner the mark turns once.
pub const MARK_SPIN: f32 = 360.;

/// The laptop's picture: 455 × 341 with its corner at (413, 163).
pub const LAPTOP: (f32, f32, f32, f32) = (413., 163., 455., 341.);
/// The moving laptop is drawn on a canvas this many times the picture's
/// size, so a turning laptop isn't cut off at the picture's edge.
pub const LAPTOP_CANVAS: f32 = 1.5;
/// Turns it makes on the way up.
pub const LAPTOP_TURNS: f32 = 1.;

/// "Welcome to Isonomic": 18 on 24, centred in a 400 column at (440, 526).
pub const TITLE: (f32, f32, f32, f32) = (440., 526., 400., 24.);
pub const TITLE_SIZE: f32 = 18.;
/// The two buttons: 320 × 44, at (480, 562) and (480, 618).
pub const BUTTON: (f32, f32) = (320., 44.);
pub const BUTTON_LEFT: f32 = 480.;
pub const BUTTON_TOPS: [f32; 2] = [562., 618.];
pub const BUTTON_PAD: (f32, f32) = (20., 6.);
pub const BUTTON_LABEL: f32 = 14.;
/// The arrow's ring and the arrow in it.
pub const RING: f32 = 32.;
pub const RING_LINE: f32 = 1.5;
pub const ARROW: f32 = 16.;

/// A button's glow: four faint colours, one to each side.
pub fn button_glow() -> Vec<BoxShadow> {
    let glow = |color: u32, x: f32, y: f32| BoxShadow {
        color: rgba(color).into(),
        offset: point(px(x), px(y)),
        blur_radius: px(16.),
        spread_radius: px(0.),
        inset: false,
    };
    vec![
        glow(0x38bdf80a, 0., -2.),
        glow(0x8b5cf60a, 10., 2.),
        glow(0xec48990a, 0., 4.),
        glow(0xe8501f0a, -10., 2.),
    ]
}

pub mod terms;

/// The order of events, in seconds from the first frame.
pub mod at {
    /// The mark's intro runs first (its own 1.5 s), then it rests a beat.
    pub const MARK_LEAVES: f32 = 1.5 + 0.25;
    /// How long the mark's sound takes to fade out. It is silent by the
    /// time the mark leaves the centre.
    pub const SOUND_FADE: f32 = 0.25;
    /// The mark turns and shrinks into its corner.
    pub const MARK_TRAVEL: f32 = 0.7;
    /// The laptop leaves the bottom edge just as the mark lands.
    pub const LAPTOP_RISES: f32 = MARK_LEAVES + MARK_TRAVEL - 0.1;
    pub const LAPTOP_FLIGHT: f32 = 1.3;
    /// Its display lights over this share of the flight, ending with it.
    pub const LIGHT_SHARE: f32 = 0.6;
    /// The landed laptop gives way to its finished picture.
    pub const SETTLE: f32 = 0.3;
    /// The title, then each button, comes up from the bottom edge.
    pub const TITLE_RISES: f32 = LAPTOP_RISES + 0.55;
    pub const BUTTONS_RISE: f32 = LAPTOP_RISES + 0.7;
    pub const BUTTON_STAGGER: f32 = 0.12;
    pub const RISE: f32 = 0.7;
    /// Nothing moves after this.
    pub const END: f32 = BUTTONS_RISE + BUTTON_STAGGER + RISE + 0.1;
}

/// Where a point of the design sits in a window drawn at `origin`.
pub fn place(origin: Point<Pixels>, x: f32, y: f32) -> Point<Pixels> {
    point(origin.x + px(x), origin.y + px(y))
}

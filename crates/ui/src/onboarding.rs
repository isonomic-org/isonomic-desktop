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

/// The laptop: 300 × 225 with its corner at (490, 150). Its top is at 150
/// on every screen; only where it is across the window changes.
pub const LAPTOP: (f32, f32, f32, f32) = (490., 150., 300., 225.424);
/// The moving laptop is drawn on a canvas this many times the picture's
/// size, so a turning laptop isn't cut off at the picture's edge.
pub const LAPTOP_CANVAS: f32 = 1.5;
/// Turns it makes on the way up.
pub const LAPTOP_TURNS: f32 = 1.;

/// "Welcome to Isonomic": 18 on 24, centred in a 400 column at (440, 397).
pub const TITLE: (f32, f32, f32, f32) = (440., 397., 400., 24.);
pub const TITLE_SIZE: f32 = 18.;
/// "Start new account": 320 × 44 at (480, 433). Under it, the link to
/// recover an account: 14 on 18, centred in the title's column at 489.
pub const BUTTON: (f32, f32) = (320., 44.);
pub const BUTTON_LEFT: f32 = 480.;
pub const BUTTON_TOP: f32 = 433.;
pub const LINK_TOP: f32 = 489.;
pub const LINK_LABEL: (f32, f32) = (14., 18.);
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

pub mod kit;
pub mod passkey;
pub mod steps;
pub mod terms;

/// The order of events, in seconds from the first frame. One thing at a
/// time: each step starts as the one before it lands.
pub mod at {
    /// The mark's intro runs first (its own 1.5 s), then it rests a beat.
    pub const MARK_LEAVES: f32 = 1.5 + 0.3;
    /// How long the mark's sound takes to fade out. It is silent by the
    /// time the mark leaves the centre.
    pub const SOUND_FADE: f32 = 0.25;
    /// The mark turns and shrinks into its corner.
    pub const MARK_TRAVEL: f32 = 0.8;
    /// The laptop leaves the bottom edge once the mark has landed.
    pub const LAPTOP_RISES: f32 = MARK_LEAVES + MARK_TRAVEL;
    pub const LAPTOP_FLIGHT: f32 = 1.4;
    /// Its display lights over this share of the flight, ending with it.
    pub const LIGHT_SHARE: f32 = 0.6;
    /// Once it has landed the title comes up from the bottom edge, then
    /// each button.
    pub const TITLE_RISES: f32 = LAPTOP_RISES + LAPTOP_FLIGHT;
    pub const BUTTONS_RISE: f32 = TITLE_RISES + 0.3;
    pub const BUTTON_STAGGER: f32 = 0.2;
    pub const RISE: f32 = 0.6;
    /// Onboarding's window closes this long after the app's has opened.
    pub const CLOSES_AFTER: f32 = 0.3;
    /// Nothing moves after this.
    pub const END: f32 = BUTTONS_RISE + BUTTON_STAGGER + RISE + 0.1;
}

/// Where a point of the design sits in a window drawn at `origin`.
pub fn place(origin: Point<Pixels>, x: f32, y: f32) -> Point<Pixels> {
    point(origin.x + px(x), origin.y + px(y))
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes, places, colours and timings of onboarding's fourth screen, from
//! the design ("Onboarding 0.4", "04 · Passkey ceremony — idle"). What it
//! shares with the third screen (the sidebar, the stepper, the footer and
//! its buttons) is in `terms`.

use isonomic_app::ui::{rgb, Hsla};

/// The Face ID seal: its tile is 120 square at (140, 464). The seal is
/// drawn in a frame the width of its wider ring of light, on a canvas this
/// many times that, and turns over this often on its way in.
pub const TILE: (f32, f32, f32) = (140., 464., 120.);
pub const SEAL_FRAME: f32 = 160.;
pub const SEAL_CANVAS: f32 = 1.5;
pub const TURNS: f32 = 1.;
/// The two rings of light around it, each: its width; the share of that
/// it shrinks to between breaths; how solid its fill and its line are at
/// full; and how much of that is left between breaths.
pub const RINGS: [(f32, f32, f32, f32, f32); 2] =
    [(136., 0.90, 0.09, 0.16, 0.35), (160., 0.82, 0.05, 0.10, 0.)];
/// The wider ring breathes this share of a breath behind the other.
pub const RING_LAG: f32 = 0.14;
pub const RING_LINE: f32 = 0.75;
pub fn light() -> Hsla {
    rgb(0x2b8bdc).into()
}

/// The content: 784 wide, 48 in from the sidebar and 40 down.
pub const CONTENT: (f32, f32, f32) = (48., 40., 784.);
/// The heading's column is 560 in the design; 8 more here, so that a
/// line that just fits there isn't broken.
pub const HEAD: f32 = 568.;
pub const HEAD_GAP: f32 = 10.;
pub const GAP: f32 = 24.;
/// The card of what happens next.
pub const CARD_PAD: (f32, f32) = (20., 16.);
pub const CARD_CORNER: f32 = 8.;
pub const CAPTION: (f32, f32) = (11., 16.);
pub const ROW_PAD: f32 = 12.;
pub const ROW_GAP: f32 = 12.;
pub const NUMBER: (f32, f32) = (12., 18.);
pub const NUMBER_PAD: (f32, f32) = (8., 2.);
pub const ROW_TITLE: (f32, f32) = (14., 20.);
pub const ROW_TEXT: (f32, f32) = (13., 20.);
pub const ROW_WIDE: f32 = 440.;
pub fn faint() -> Hsla {
    rgb(0x6b7280).into()
}
pub fn chip() -> Hsla {
    rgb(0xf3f4f6).into()
}
/// "Why no password?": 17 either side of its label.
pub const ASIDE_PAD: f32 = 17.;

/// The order of events, in seconds from the end of the signing. One thing
/// at a time.
pub mod at {
    /// The next panel slides over the last from the right, and the
    /// stepper's words change under way.
    pub const SLIDE: f32 = 0.7;
    pub const WORDS_CHANGE: f32 = 0.3;
    /// The seal turns over as it comes out from under the panel.
    pub const SEAL_FLIES: f32 = SLIDE - 0.1;
    pub const FLIGHT: f32 = 1.;
    /// The dotted line comes back, and the rings of light come up and
    /// breathe for as long as the screen is up.
    pub const LIGHT_SHOWS: f32 = SEAL_FLIES + FLIGHT - 0.1;
    pub const LIGHT_FADE: f32 = 0.4;
    pub const BREATH: f32 = 2.;
    /// Nothing but the light and the dots moves after this.
    pub const END: f32 = LIGHT_SHOWS + LIGHT_FADE + 0.1;
}

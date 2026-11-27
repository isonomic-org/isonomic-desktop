// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes, places, colours and timings of onboarding's third screen, from
//! the design ("Onboarding 0.4", "03 · Disclosures and acceptance").
//! Places are in the window's own 1280 × 800, from its top left.

use isonomic_app::ui::{point, px, rgb, rgba, BoxShadow, Hsla};

/// The sidebar: 400 wide, the full height, with a hairline on its right.
pub const SIDEBAR: f32 = 400.;
pub fn sidebar() -> Hsla {
    rgb(0xf9fafb).into()
}
/// Text one step quieter than the ink, and the disclosure's own grey.
pub fn quiet() -> Hsla {
    rgb(0x4b5563).into()
}
pub fn body() -> Hsla {
    rgb(0x374151).into()
}
pub fn outline() -> Hsla {
    rgb(0xd1d5db).into()
}

/// The bundled families the step label and the disclosure are set in.
pub const MONO: &str = "IBM Plex Mono";
pub const SERIF: &str = "IBM Plex Serif";

/// The laptop's picture in the sidebar: 300 wide at (50, 150).
pub const LAPTOP: (f32, f32, f32, f32) = (50., 150., 300., 225.424);
/// The clipboard: 118.8 × 158.4 at (141, 456), seen from above. Its board
/// fills this share of that width.
pub const CLIPBOARD: (f32, f32, f32, f32) = (141., 456., 118.8, 158.4);
pub const CLIPBOARD_FILL: f32 = 0.958;
pub const CLIPBOARD_ELEVATION: f32 = 89.9;
/// The pen beside it: its box, the length it is drawn at, how far it
/// leans, and the square it is drawn in, about the box's middle.
pub const PEN: (f32, f32, f32, f32) = (273.8, 474.4, 55., 141.);
pub const PEN_LENGTH: f32 = 143.;
pub const PEN_LEAN: f32 = -15.;
pub const PEN_CANVAS: f32 = 168.;
/// The clipboard is drawn on a canvas this many times its box, so that
/// turning over it isn't cut off.
pub const CLIPBOARD_CANVAS: f32 = 1.5;
/// A prop starts with its box this far under the content's edge.
pub const TUCK: f32 = 20.;
/// Turns each makes on its way in.
pub const TURNS: f32 = 1.;

/// The dotted line from the laptop to the clipboard: a port, then fourteen
/// dots straight down.
pub const DOTS_X: f32 = 200.;
pub const DOTS_TOP: f32 = 330.34;
pub const DOTS_STEP: f32 = 10.2965;
pub const DOTS: usize = 14;
pub const DOT: f32 = 2.25;
pub const PORT: f32 = 5.5;
pub fn dot() -> Hsla {
    rgb(0x5c5e66).into()
}
/// A dot at rest is this faint; the pulse brings it to full and grows it.
pub const DOT_REST: f32 = 0.3;
pub const DOT_SWELL: f32 = 0.9;
/// How many dots long the pulse is.
pub const PULSE_LENGTH: f32 = 4.;

/// The stepper: 320 wide at (40, 638).
pub const STEPPER: (f32, f32, f32) = (40., 638., 320.);
pub const STEPPER_TEXT: f32 = 241.;
pub const STEPS: usize = 6;
pub const STEP_BAR: f32 = 6.;
pub const STEP_LABEL: (f32, f32) = (12., 16.);
pub const STEP_TITLE: (f32, f32) = (20., 28.);

/// The content: a header, the disclosure under it, and an 80 high footer.
pub const PAD: f32 = 48.;
pub const HEADER_PAD: (f32, f32) = (40., 24.);
pub const HEADING: (f32, f32) = (32., 40.);
pub const SUBHEADING: (f32, f32) = (15., 24.);
pub const DISCLOSURE: (f32, f32) = (159., 561.);
pub const DISCLOSURE_PAD: (f32, f32) = (24., 32.);
pub const CLAUSE_TITLE: (f32, f32) = (16., 24.);
pub const CLAUSE: (f32, f32) = (14., 23.);
pub const CLAUSE_GAP: f32 = 12.;
pub const FOOTER: f32 = 80.;
pub const FOOTER_PAD: f32 = 32.;
/// A footer button: 14 on 20, 9 above and below, with a 20 arrow.
pub const ACTION_LABEL: (f32, f32) = (14., 20.);
pub const ACTION_PAD: (f32, f32, f32) = (15., 17., 9.);
pub const ACTION_CORNER: f32 = 6.;
pub const ACTION_ARROW: f32 = 20.;
pub fn action_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: rgba(0x0000000d).into(),
        offset: point(px(0.), px(1.)),
        blur_radius: px(2.),
        spread_radius: px(0.),
        inset: false,
    }]
}

/// The order of events, in seconds from the press on "Start new account".
/// One thing at a time: each step starts as the one before it lands.
pub mod at {
    /// "Recover your account" and the title drop out, then "Start new
    /// account" leaves to the right.
    pub const LEAVE: f32 = 0.4;
    pub const LEAVE_STAGGER: f32 = 0.1;
    /// The laptop turns over on its way to the sidebar, whose colour
    /// comes in under it.
    pub const LAPTOP_MOVES: f32 = 0.45;
    pub const LAPTOP_MOVE: f32 = 1.2;
    pub const SIDEBAR_SHOWS: f32 = LAPTOP_MOVES;
    pub const SIDEBAR_FADE: f32 = 0.8;
    /// The content slides in from the right.
    pub const CONTENT_SLIDES: f32 = LAPTOP_MOVES + LAPTOP_MOVE - 0.05;
    pub const SLIDE: f32 = 0.7;
    /// The stepper comes up from the bottom edge.
    pub const STEPPER_RISES: f32 = CONTENT_SLIDES + SLIDE - 0.1;
    pub const RISE: f32 = 0.6;
    /// The clipboard and the pen turn over as they come out from under
    /// the content, as a pair, the pen a step behind.
    pub const CLIPBOARD_FLIES: f32 = STEPPER_RISES + RISE - 0.1;
    pub const PEN_FLIES: f32 = CLIPBOARD_FLIES + 0.15;
    pub const FLIGHT: f32 = 1.;
    /// The dots come out one by one, then pulse toward the clipboard for
    /// as long as the screen is up.
    pub const DOTS_START: f32 = PEN_FLIES + FLIGHT - 0.1;
    pub const DOT_STAGGER: f32 = 0.04;
    pub const DOT_SHOWS: f32 = 0.2;
    pub const PULSE: f32 = 1.6;
    /// Nothing but the dots moves after this.
    pub const END: f32 = PEN_FLIES + FLIGHT + 0.1;
}

/// The signature the pen writes on the last page.
pub fn signature() -> Hsla {
    rgb(0x1a2238).into()
}
pub const SIGNATURE_LINE: f32 = 1.1;

/// The order of events, in seconds from the press on "Accept and
/// continue". One thing at a time.
pub mod signs {
    /// Going back, the signing is undone this many times as fast as it
    /// was done.
    pub const UNDO_RATE: f32 = 4.;
    /// The pages lift by their corner and curl back off the last page.
    pub const LIFT: f32 = 1.;
    /// The pen goes to the signature's start, writes it, and goes back.
    pub const PEN_REACHES: f32 = LIFT - 0.1;
    pub const REACH: f32 = 0.5;
    pub const WRITES: f32 = PEN_REACHES + REACH;
    pub const WRITE: f32 = 1.5;
    pub const PEN_RETURNS: f32 = WRITES + WRITE + 0.15;
    /// The pages are let go and settle.
    pub const RELEASES: f32 = PEN_RETURNS + REACH - 0.1;
    pub const RELEASE: f32 = 0.8;
    /// The clipboard turns over and leaves by the left edge; then the pen.
    pub const CLIPBOARD_LEAVES: f32 = RELEASES + RELEASE;
    pub const LEAVE: f32 = 0.9;
    pub const PEN_LEAVES: f32 = CLIPBOARD_LEAVES + LEAVE - 0.25;
    pub const PEN_LEAVE: f32 = 0.8;
    /// The dotted line fades as the clipboard goes.
    pub const DOTS_FADE: f32 = 0.3;
    /// Onboarding hands over.
    pub const DONE: f32 = PEN_LEAVES + PEN_LEAVE + 0.1;
}

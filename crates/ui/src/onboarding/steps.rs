// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes, colours and timings shared by onboarding's later screens, from
//! the design ("Onboarding 0.4", "05" to "12"): the Face ID sheet, the
//! steps from "Your account exists." to "Finish", and "Setting up your
//! workspace". What they share with the third and fourth screens is in
//! `terms` and `passkey`.

use isonomic_app::ui::{rgb, rgba, Hsla};

/// Each step's art sits about this line down the sidebar.
pub const ART_X: f32 = 200.;
/// A sprite is never drawn thinner than this share of its width as it
/// turns over.
pub const EDGE_ON: f32 = 0.03;

/// "Your account exists.": the card wallet and, beside it, the ID card.
/// Before they leave, the card goes up over the wallet's mouth and slips
/// in: where its corner is over the wallet, and where it comes to rest in
/// it.
pub const WALLET: (f32, f32, f32, f32) = (82., 464., 117.5, 120.);
pub const CARD: (f32, f32, f32, f32) = (211.5, 477., 106.25, 93.75);
pub const CARD_OVER: (f32, f32) = (87.6, 407.);
pub const CARD_IN: (f32, f32) = (87.6, 470.);

/// "Turn on local AI models": the drives are drawn as the design draws
/// them, the way an icon of a drive is: from in front and this many
/// degrees up, so the top shows with the front edge and its light under
/// it, and without narrowing toward the back. The boxes are the design's
/// drives without their shadows, back to front: the outer pair, the
/// inner pair, the middle one.
pub const DRIVE_ELEVATION: f32 = 51.;
pub const DRIVES: [(f32, f32, f32, f32); 5] = [
    (-11.99, 491.45, 53.71, 65.65),
    (358.05, 491.45, 53.71, 65.65),
    (59.08, 482.36, 69.68, 85.16),
    (270.93, 482.36, 69.68, 85.16),
    (148.97, 464.19, 101.61, 124.2),
];

/// A list row, its mark and the gap after it, its chip and its switch.
pub const ROW: f32 = 63.;
pub const LOGO: f32 = 40.;
pub const LOGO_GAP: f32 = 16.;
pub const CHIP: f32 = 28.;
pub const SWITCH: (f32, f32, f32) = (28., 16., 12.);
pub const SMALL: (f32, f32) = (12., 18.);
pub const FIELD: (f32, f32) = (360., 38.);
pub const WORD: f32 = 36.;
pub const TABS: f32 = 40.;

pub fn warning() -> Hsla {
    rgb(0xfffbeb).into()
}
pub fn warning_ink() -> Hsla {
    rgb(0x92400e).into()
}
pub fn good() -> Hsla {
    rgb(0xdcfce7).into()
}
pub fn good_ink() -> Hsla {
    rgb(0x166534).into()
}
pub fn wash() -> Hsla {
    rgb(0xf3f4f6).into()
}
/// What dims the window behind the Face ID sheet.
pub fn dim() -> Hsla {
    rgba(0x11182766).into()
}

/// The Face ID sheet: 360 wide, in the middle of the window.
pub const SHEET: (f32, f32) = (360., 250.);
pub const SHEET_CORNER: f32 = 12.;

/// "Setting up your workspace": the laptop in the middle, 490 across and
/// as far down as on every other screen, its name under it, and the pill
/// under that. The design has all three 85 lower.
pub const SETUP_LAPTOP: f32 = 490.;
pub const SETUP_NAME: (f32, f32, f32, f32) = (440., 397., 400., 24.);
pub const SETUP_PILL: (f32, f32, f32, f32) = (480., 437., 320., 44.);
/// The laptop turns over this often on its way there.
pub const SETUP_TURNS: f32 = 1.;

/// The order of events on each step, in seconds from the press that
/// brought it on. One thing at a time.
pub mod at {
    /// On leaving "Your account exists.", the card first slips into the
    /// wallet; everything else waits for it.
    pub const SLIP: f32 = 0.9;
    /// The last step's art turns over and leaves by the left edge while
    /// the next panel slides over the last from the right.
    pub const LEAVE: f32 = 0.6;
    pub const SLIDE: f32 = 0.7;
    pub const WORDS_CHANGE: f32 = 0.3;
    /// The step's art turns over as it comes out from under the panel.
    pub const ART_ARRIVES: f32 = SLIDE - 0.1;
    pub const FLIGHT: f32 = 0.9;
    /// The dotted line fades with the old art and comes back with the new.
    pub const DOTS_FADE: f32 = 0.3;
    /// Nothing but the dots moves after this.
    pub const END: f32 = ART_ARRIVES + FLIGHT + 0.1;

    /// The Face ID sheet comes up, stays while the system would be asked,
    /// and goes.
    pub const SHEET_FADE: f32 = 0.25;
    pub const ASKING: f32 = 1.8;

    /// "Setting up your workspace": everything clears, the laptop turns
    /// over on its way to the middle, its name and the pill come up, and onboarding hands
    /// over.
    pub const CLEAR: f32 = 0.4;
    pub const LAPTOP_CENTRES: f32 = 0.3;
    pub const CENTRE: f32 = 1.2;
    pub const PILL_RISES: f32 = LAPTOP_CENTRES + CENTRE - 0.1;
    pub const RISE: f32 = 0.6;
    pub const SETTING_UP: f32 = 3.5;
}

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

/// "Your account exists.": the card wallet in the middle and the ID card
/// lying over it, its top out past the wallet's mouth. Before they
/// leave, the card turns over as it lifts clear of the mouth, then tucks
/// in: where its corner is when it is clear, where it comes to rest in
/// the wallet, and how far past that it dips on the way.
pub const WALLET: (f32, f32, f32, f32) = (141.25, 464., 117.5, 120.);
pub const CARD: (f32, f32, f32, f32) = (146.875, 452., 106.25, 93.75);
pub const CARD_OVER: (f32, f32) = (146.875, 398.);
pub const CARD_IN: (f32, f32) = (146.875, 470.);
pub const CARD_DIP: f32 = 5.;

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

/// The design's buttons: how tall one is, the space beside its label,
/// and its icon; and the smaller "Connect".
pub const BUTTON: f32 = 38.;
pub const BUTTON_PAD: f32 = 16.;
pub const ICON: f32 = 20.;
pub const CONNECT: f32 = 34.;
pub const CONNECT_PAD: f32 = 12.;
/// The passkey's mark in its tile.
pub const PASSKEY_ICON: f32 = 22.;
/// What a field and a note lie on.
pub fn field() -> Hsla {
    rgb(0xf9fafb).into()
}

/// A list row, its mark and the gap after it.
pub const ROW: f32 = 63.;
pub const LOGO: f32 = 40.;
pub const LOGO_GAP: f32 = 16.;
/// The field in a connector's row for its token, or a custom server's
/// URL or command; and the "i" in a model's pill.
pub const ROW_FIELD: f32 = 220.;
pub const PILL_ICON: f32 = 14.;
pub const SMALL: (f32, f32) = (12., 18.);
pub const FIELD: (f32, f32) = (360., 38.);

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
pub const SHEET: (f32, f32) = (360., 297.);
/// Its padding above and to the sides, and below; the gap between its
/// lines; its icon and its spinner; and its three sizes of text.
pub const SHEET_PAD: (f32, f32) = (28., 22.);
pub const SHEET_GAP: f32 = 14.;
pub const SHEET_ICON: f32 = 44.;
pub const SHEET_SPINNER: f32 = 32.;
pub const SHEET_TITLE: (f32, f32) = (16., 22.);
pub const SHEET_TEXT: (f32, f32) = (13., 19.);
pub const SHEET_STATUS: (f32, f32) = (14., 20.);
/// How dark the window behind it gets.
pub const SHEET_DIM: f32 = 0.4;
/// "Your account exists.": the gap between its cards, a row of what the
/// address is for, the address box's padding around its button, and how
/// wide its note and a setting's words run.
pub const CARD_GAP: f32 = 22.;
pub const ROOT_ROW_PAD: f32 = 8.5;
pub const ADDRESS_PAD: f32 = 5.;
pub const NOTE_WIDE: f32 = 520.;
pub const SETTING_WIDE: f32 = 420.;
/// A step with no rule under its heading starts this far under it.
pub const BODY_TOP: f32 = 22.;
/// A note: its icon, its title and its words.
pub const NOTE_ICON: f32 = 20.;
pub const NOTE_TITLE: (f32, f32) = (13., 20.);
pub const NOTE_TEXT: (f32, f32) = (12., 19.);
/// A list with nothing to show says so this far down.
pub const EMPTY_PAD: f32 = 48.;
/// How strong the ring around the pill's mark is.
pub const SETUP_RING: f32 = 0.5;
pub const SHEET_CORNER: f32 = 12.;

/// "Setting up your workspace": the laptop in the middle, 490 across and
/// as far down as on every other screen, its name under it, and the pill
/// under that. The design has all three 85 lower.
pub const SETUP_LAPTOP: f32 = 490.;
pub const SETUP_NAME: (f32, f32, f32, f32) = (440., 397., 400., 24.);
pub const SETUP_PILL: (f32, f32, f32, f32) = (480., 437., 320., 44.);
/// What is said under the pill: how wide it may be, and how far under.
pub const SETUP_SAYS: (f32, f32) = (480., 14.);
/// The pill's mark and its spinner, and the colour of both.
pub const SETUP_MARK: (f32, f32) = (32., 12.);
pub fn accent() -> Hsla {
    rgb(0xe8501f).into()
}
/// The laptop turns over this often on its way there.
pub const SETUP_TURNS: f32 = 1.;

/// The order of events on each step, in seconds from the press that
/// brought it on. One thing at a time.
pub mod at {
    /// "Your account exists." comes on wallet first; the card follows it
    /// out from under the panel this much later.
    pub const CARD_FOLLOWS: f32 = 0.45;
    /// On leaving it, the card first turns over as it lifts, tucks into
    /// the wallet, and rests there a moment; everything else waits.
    pub const CARD_LIFTS: f32 = 0.55;
    pub const CARD_TUCKS: f32 = 0.6;
    pub const CARD_RESTS: f32 = 0.25;
    pub const SLIP: f32 = CARD_LIFTS + CARD_TUCKS + CARD_RESTS;
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

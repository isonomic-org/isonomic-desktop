// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes and timings of the recovery kit's holder and its padlock, which
//! "Save your recovery kit" and "Confirm your recovery kit" share in the
//! sidebar, and of the words typed back on the second of them (design
//! "Onboarding 0.4", "08", "08b" and "08c").

use isonomic_app::ui::{rgb, Hsla};

/// The canvas the holder is drawn on, and where its middle is in the
/// window. The canvas is wider and taller than the holder so that it can
/// turn over and fold inside it.
pub const CANVAS: (f32, f32) = (220., 300.);
pub const CENTRE: (f32, f32) = (200., 540.);
/// How tall the open holder is, loop to loop.
pub const OPEN_TALL: f32 = 156.;
/// The padlock's canvas.
pub const LOCK_CANVAS: (f32, f32) = (48., 84.);
/// A wrong word shakes the holder this far to each side, this many times.
pub const SHAKE: (f32, f32) = (7., 4.);

/// The words to type back: a cell of the grid, the gaps between cells,
/// and the words in a row.
pub const CELL: f32 = 38.;
pub const CELL_GAP: f32 = 10.;
/// A cell to type in is this wide, a quarter of the row less its gaps;
/// the words not asked for are this faint.
pub const CELL_WIDE: f32 = 188.5;
pub const SHOWN: f32 = 0.45;
pub const ACROSS: usize = 4;
pub fn wrong() -> Hsla {
    rgb(0xb91c1c).into()
}
pub fn focus() -> Hsla {
    rgb(0x2563eb).into()
}

/// The order of events, in seconds.
pub mod at {
    use crate::onboarding::steps::at as step;

    /// "Save your recovery kit": the holder comes out from under the
    /// panel closed, as every step's art does; then it opens; then its
    /// letters turn into place one after another.
    pub const OPENS: f32 = step::ART_ARRIVES + step::FLIGHT;
    pub const OPEN: f32 = 0.9;
    pub const SPINS: f32 = OPENS + OPEN - 0.15;
    pub const SPIN: f32 = 0.5;
    pub const SPIN_STAGGER: f32 = 0.012;
    /// "Confirm your recovery kit": it turns its back.
    pub const TURNS_AWAY: f32 = 0.2;
    pub const TURN: f32 = 0.9;
    /// A wrong word: it shakes.
    pub const SHAKE: f32 = 0.45;
    /// The right words, from the press: it closes and lies with its mark
    /// up, the padlock comes out from under the panel, its body snaps up
    /// onto its shackle, and after a moment the next step comes on.
    pub const CLOSE: f32 = 1.;
    pub const LOCK_COMES: f32 = CLOSE + 0.1;
    pub const LOCK_FLIGHT: f32 = 0.6;
    pub const LOCK_CLIPS: f32 = LOCK_COMES + LOCK_FLIGHT + 0.05;
    pub const CLIP: f32 = 0.22;
    pub const CONFIRMED: f32 = LOCK_CLIPS + CLIP + 0.45;

    // One thing at a time.
    const _: () = assert!(LOCK_COMES >= CLOSE && LOCK_CLIPS >= LOCK_COMES + LOCK_FLIGHT);
}

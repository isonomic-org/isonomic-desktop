// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sizes of the Settings pages, from the design's Settings 0.3.

use isonomic_app::ui::{px, Pixels};

/// Width of the list of pages.
pub fn sidebar_width() -> Pixels {
    px(236.)
}

/// The space above each section of the list of pages after the first.
pub fn section_gap() -> Pixels {
    px(12.)
}

/// Width of a page's column of settings.
pub fn column_width() -> Pixels {
    px(720.)
}

/// The least height of a row.
pub fn row_height() -> Pixels {
    px(56.)
}

/// A row's leading tile, square.
pub fn lead_tile() -> Pixels {
    px(36.)
}

/// A brand's mark inside the leading tile, square.
pub fn lead_mark() -> Pixels {
    px(20.)
}

/// A search field beside a page's tabs.
pub fn find_width() -> Pixels {
    px(240.)
}

/// A field in a row when the design gives it no width.
pub fn field_width() -> Pixels {
    px(260.)
}

/// A slider in a row.
pub fn slider_width() -> Pixels {
    px(160.)
}

/// Height of a chart's bars with their axis.
pub fn chart_height() -> Pixels {
    px(180.)
}

/// Height of the lines of a service's log.
pub fn log_height() -> Pixels {
    px(240.)
}

/// A companion's card: three to a row of the column.
pub fn card_width() -> Pixels {
    px(232.)
}

/// The square that holds a companion's glyph.
pub fn glyph() -> Pixels {
    px(56.)
}

/// One word of a recovery kit, with its number: three to a row.
pub fn word_width() -> Pixels {
    px(144.)
}

/// A dialog when the design gives it no width.
pub fn dialog_width() -> Pixels {
    px(480.)
}

/// A length the design gives in points: a field's or a dialog's width, a
/// text area's height, a table column's width.
pub fn points(points: f64) -> Pixels {
    px(points as f32)
}

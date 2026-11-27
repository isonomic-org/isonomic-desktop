// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The blocks that show rather than set: a table, a chart, what the disk
//! holds, and the pieces only a dialog has.

use serde_json::Value;
use ui::base::{h_flex, v_flex};
use ui::component::chart::BarChart;
use ui::component::progress::{BarSegment, SegmentedBar};
use ui::component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::prelude::FluentBuilder as _;
use ui::settings as sizes;
use ui::{
    div, tokens, AnyElement, App, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _,
};

use super::row::panel;
use super::spec::{list, text};

/// A table: its first columns as wide as the design draws them, the last
/// taking the rest.
pub fn table(of: &'static Value, cx: &App) -> AnyElement {
    let widths = list(of, "widths");
    let last = widths.len().saturating_sub(1);
    let width = move |at: usize| {
        let width = widths.get(at).and_then(Value::as_f64);
        width.filter(|_| at < last).map(sizes::points)
    };
    let heads = list(of, "head").iter().enumerate().map(|(at, head)| {
        let head = TableHead::new().child(head.as_str().unwrap_or(""));
        match width(at) {
            Some(width) => head.w(width),
            None => head,
        }
    });
    let rows = list(of, "rows").iter().map(|row| {
        let cells = row.as_array().map_or(&[][..], Vec::as_slice);
        TableRow::new().children(cells.iter().enumerate().map(|(at, cell)| {
            let cell = TableCell::new().child(cell.as_str().unwrap_or(""));
            match width(at) {
                Some(width) => cell.w(width),
                None => cell,
            }
        }))
    });
    let theme = cx.theme();
    Table::new()
        .border_1()
        .border_color(theme.border)
        .rounded(theme.radius)
        .child(TableHeader::new().child(TableRow::new().children(heads)))
        .child(TableBody::new().children(rows))
        .into_any_element()
}

/// A bar for each day, under the chart's title.
pub fn chart(sid: &str, index: usize, of: &'static Value, cx: &App) -> AnyElement {
    let bands = list(of, "bands");
    let bars = list(of, "bars").iter().enumerate().map(|(at, bar)| {
        let band = bands.get(at).and_then(Value::as_str).unwrap_or("");
        (SharedString::from(band), bar.as_f64().unwrap_or(0.))
    });
    let bars: Vec<(SharedString, f64)> = bars.collect();
    let theme = cx.theme();
    let (fill, muted) = (theme.chart_1, theme.muted_foreground);
    let head = h_flex()
        .justify_between()
        .child(div().text_sm().font_medium().child(text(of, "title")))
        .child(div().text_sm().text_color(muted).child(text(of, "aside")));
    let chart = BarChart::new(bars)
        .band(|bar: &(SharedString, f64)| bar.0.clone())
        .value(|bar: &(SharedString, f64)| bar.1)
        .fill(move |_, _, _, _| fill);
    panel(format!("{sid}-chart-{index}"), cx)
        .child(
            v_flex()
                .p_4()
                .gap_3()
                .child(head)
                .child(div().h(sizes::chart_height()).child(chart)),
        )
        .into_any_element()
}

/// What the disk holds: how much is used and free, a segment for each
/// kind, and each kind's name and size under its colour.
pub fn storage(of: &'static Value, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let muted = theme.muted_foreground;
    let colors = [theme.chart_1, theme.chart_2, theme.chart_3, theme.chart_4];
    let parts = list(of, "parts");
    let whole: f64 = parts.iter().filter_map(Value::as_f64).sum();
    let kinds = list(of, "legend").iter().enumerate().map(|(at, kind)| {
        let part = parts.get(at).and_then(Value::as_f64).unwrap_or(0.);
        let (name, size) = (
            kind[0].as_str().unwrap_or(""),
            kind[1].as_str().unwrap_or(""),
        );
        (name, size, part, colors[at % colors.len()])
    });
    let kinds: Vec<_> = kinds.collect();
    let bar = SegmentedBar::new("settings-storage", whole)
        .label(text(of, "used"))
        .value_text(text(of, "free"))
        .segments(
            kinds
                .iter()
                .map(|(name, _, part, color)| BarSegment::new(*name, *part, *color)),
        )
        .w_full();
    let legend = kinds.iter().map(|(name, size, _, color)| {
        let dot = div().size(tokens::status_dot()).rounded_full().bg(*color);
        let name = div().text_xs().text_color(muted).child(*name);
        v_flex()
            .gap_1()
            .child(h_flex().gap_1p5().items_center().child(dot).child(name))
            .child(div().text_sm().child(*size))
    });
    v_flex()
        .w_full()
        .gap_3()
        .child(
            h_flex()
                .justify_between()
                .child(div().text_sm().font_medium().child(text(of, "used")))
                .child(div().text_sm().text_color(muted).child(text(of, "free"))),
        )
        .child(bar)
        .child(h_flex().gap_6().children(legend))
        .into_any_element()
}

/// The places of a recovery kit's words, numbered. The words themselves
/// are the engine's to give and are not shown here.
pub fn words(of: &'static Value, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let (edge, muted, round) = (theme.border, theme.muted_foreground, theme.radius);
    let mono = theme.mono_font_family.clone();
    let count = of["count"].as_u64().unwrap_or(0);
    h_flex()
        .flex_wrap()
        .gap_2()
        .children((1..=count).map(|place| {
            h_flex()
                .w(sizes::word_width())
                .gap_2()
                .px_3()
                .py_2()
                .rounded(round)
                .border_1()
                .border_color(edge)
                .child(div().text_xs().text_color(muted).child(place.to_string()))
                .child(div().text_sm().font_family(mono.clone()).child("•••••"))
        }))
        .into_any_element()
}

/// A code or a key press written large, with a line about it under it.
pub fn written(of: &'static Value, key: &str, cx: &App) -> AnyElement {
    let theme = cx.theme();
    v_flex()
        .w_full()
        .items_center()
        .gap_2()
        .py_6()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .child(
            div()
                .text_2xl()
                .font_family(theme.mono_font_family.clone())
                .child(text(of, key)),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(text(of, "text")),
        )
        .into_any_element()
}

/// Where a file is dropped.
pub fn dropzone(of: &'static Value, cx: &App) -> AnyElement {
    let theme = cx.theme();
    v_flex()
        .w_full()
        .items_center()
        .gap_1()
        .py_8()
        .rounded(theme.radius)
        .border_1()
        .border_dashed()
        .border_color(theme.border)
        .child(div().text_sm().font_medium().child(text(of, "title")))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(text(of, "text")),
        )
        .into_any_element()
}

/// The last lines of a service's log.
pub fn log(sid: &str, of: &'static Value, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let lines = text(of, "text").lines();
    v_flex()
        .id(SharedString::from(format!("{sid}-log")))
        .w_full()
        .h(sizes::log_height())
        .overflow_y_scroll()
        .p_3()
        .rounded(theme.radius)
        .bg(theme.muted)
        .font_family(theme.mono_font_family.clone())
        .text_xs()
        .children(lines.map(|line| div().child(line)))
        .when(text(of, "text").is_empty(), |this| {
            this.child("No lines yet.")
        })
        .into_any_element()
}

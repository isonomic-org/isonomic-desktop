// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! A group of rows: the design's panel, each row a title, a line about it
//! and its controls.

use serde_json::Value;
use ui::base::{h_flex, v_flex};
use ui::component::avatar::Avatar;
use ui::component::group_box::{GroupBox, GroupBoxVariants as _};
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::prelude::FluentBuilder as _;
use ui::settings as sizes;
use ui::{
    div, img, AnyElement, App, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled as _, Window,
};

use super::spec::{self, list, text};
use super::{marks, Settings};

/// The design's panel: filled, with a hairline round it and its rows edge
/// to edge.
pub(super) fn panel(id: impl Into<SharedString>, cx: &App) -> GroupBox {
    let edge = StyleRefinement::default()
        .p_0()
        .gap_0()
        .border_1()
        .border_color(cx.theme().border)
        .overflow_hidden();
    GroupBox::new().id(id.into()).fill().content_style(edge)
}

/// A row's leading tile, with an icon or a maker's mark; a person's
/// initials stand alone.
fn lead(of: &'static Value, cx: &App) -> Option<AnyElement> {
    if let Some(initials) = of["avatar"].as_str() {
        return Some(Avatar::new().name(initials).into_any_element());
    }
    let mark = if let Some(name) = of["icon"].as_str() {
        marks::icon(name).into_any_element()
    } else {
        let name = of["logo"].as_str()?;
        match marks::logo(name) {
            Some(picture) => img(picture).size(sizes::lead_mark()).into_any_element(),
            // No picture of this mark yet: its initial stands in.
            None => div()
                .text_sm()
                .font_medium()
                .child(name.chars().take(1).collect::<String>().to_uppercase())
                .into_any_element(),
        }
    };
    let theme = cx.theme();
    let tile = div()
        .size(sizes::lead_tile())
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(mark);
    Some(tile.into_any_element())
}

impl Settings {
    /// Whether `row` is on show: not taken off, and holding what the
    /// page's own search asks for.
    fn shows(&self, sid: &str, row: &'static Value, query: &str) -> bool {
        if self.gone.contains(&format!("{sid}|{}", text(row, "id"))) {
            return false;
        }
        let holds = |key| text(row, key).to_lowercase().contains(query);
        query.is_empty() || holds("title") || holds("desc")
    }

    /// The group `of`, block `index` of `sid`; nothing when no row shows.
    pub(super) fn group(
        &mut self,
        sid: &'static str,
        index: usize,
        of: &'static Value,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let rows: Vec<&'static Value> = list(of, "rows")
            .iter()
            .filter(|row| self.shows(sid, row, query))
            .collect();
        let last = rows.len().checked_sub(1)?;
        let mut lines = Vec::new();
        for (at, row) in rows.into_iter().enumerate() {
            lines.push(self.row(sid, row, at == last, window, cx));
        }
        let group = panel(format!("{sid}-group-{index}"), cx).children(lines);
        Some(group.into_any_element())
    }

    fn row(
        &mut self,
        sid: &'static str,
        of: &'static Value,
        last: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let name = text(of, "id");
        let key = format!("{sid}|{name}");
        let controls = self.controls(sid, name, &key, list(of, "ctl"), window, cx);
        let theme = cx.theme();
        let (edge, muted, hover) = (theme.border, theme.muted_foreground, theme.muted);
        let (red, mono) = (theme.danger, theme.mono_font_family.clone());
        let about = text(of, "desc");
        let title = div()
            .text_sm()
            .font_medium()
            .when(of["red"] == true, |this| this.text_color(red))
            .when(of["title_mono"] == true, |this| {
                this.font_family(mono.clone())
            })
            .child(text(of, "title"));
        let words = v_flex()
            .flex_1()
            .min_w_0()
            .child(title)
            .when(!about.is_empty(), |this| {
                let about = div().text_sm().text_color(muted).child(about);
                this.child(about.when(of["desc_mono"] == true, |this| this.font_family(mono)))
            });
        h_flex()
            .id(SharedString::from(key))
            .w_full()
            .min_h(sizes::row_height())
            .px_4()
            .py_3()
            .gap_3()
            .items_center()
            .when(!last, |this| this.border_b_1().border_color(edge))
            .children(lead(&of["lead"], cx))
            .child(words)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .flex_shrink_0()
                    .children(controls),
            )
            // A row with an arrow at its end leads to a page of its own.
            .when_some(spec::link(sid, name, ">"), |this, target| {
                this.cursor_pointer()
                    .hover(move |this| this.bg(hover))
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.follow(target, window, cx)),
                    )
            })
            .into_any_element()
    }
}

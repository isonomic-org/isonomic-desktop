// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! A page's blocks, top to bottom, with the design's space between them.

use serde_json::Value;
use ui::base::{h_flex, v_flex};
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle};
use ui::component::input::{Input, InputState, Textarea};
use ui::component::tab::{Tab, TabBar};
use ui::component::theme::ActiveTheme as _;
use ui::component::Sizable as _;
use ui::component::StyledExt as _;
use ui::prelude::FluentBuilder as _;
use ui::settings as sizes;
use ui::{
    div, AnyElement, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};

use super::row::panel;
use super::spec::{self, list, text};
use super::{figures, marks, Settings};

/// `block` under the block before it, the design's distance apart: a
/// section's title stands off from what is above it and close to its
/// own panel.
fn spaced(before: &str, kind: &str, block: AnyElement) -> AnyElement {
    let holder = div().w_full().child(block);
    match (before, kind) {
        ("", _) => holder,
        (_, "section") => holder.mt_8(),
        ("header", _) => holder.mt_6(),
        ("tabs", _) => holder.mt_5(),
        ("section", _) => holder.mt_3(),
        (_, "note") => holder.mt_2(),
        _ => holder.mt_4(),
    }
    .into_any_element()
}

impl Settings {
    /// The search field of the page `named`, made the first time it is
    /// drawn. A page with tabs has one for all of them.
    fn find(
        &mut self,
        named: &str,
        hint: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        self.input(&format!("{named}|find"), hint, true, window, cx)
    }

    /// What the page's own search holds, in lower case.
    fn finding(&self, sid: &str, cx: &Context<Self>) -> String {
        let field = |named: &str| self.inputs.get(&format!("{named}|find"));
        let field = field(sid).or_else(|| field(spec::owner(sid)));
        field.map_or_else(String::new, |field| {
            field.read(cx).value().trim().to_lowercase()
        })
    }

    /// Every block of the page `of`, spaced.
    pub(super) fn page_view(
        &mut self,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let sid = text(of, "id");
        let query = self.finding(sid, cx);
        let (mut drawn, mut before) = (Vec::new(), "");
        for (index, block) in list(of, "blocks").iter().enumerate() {
            let kind = text(block, "k");
            if let Some(block) = self.block(sid, index, block, &query, window, cx) {
                drawn.push(spaced(before, kind, block));
                before = kind;
            }
        }
        drawn
    }

    /// Block `index` of the page or dialog `sid`.
    pub(super) fn block(
        &mut self,
        sid: &'static str,
        index: usize,
        of: &'static Value,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let kind = text(of, "k");
        let key = format!("{sid}|{kind}{index}");
        Some(match kind {
            "header" => self.heading(sid, "header", &key, of, window, cx),
            "section" => self.heading(sid, "section", &key, of, window, cx),
            "group" => return self.group(sid, index, of, query, window, cx),
            "tabs" => self.tabs(sid, of, window, cx),
            "back" => self.back(of, cx),
            "note" => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(text(of, "text"))
                .into_any_element(),
            "empty" => self.empty(sid, &key, of, window, cx),
            "filter" => self.filter(sid, &key, of, window, cx),
            "textarea" => {
                let state = self.area(&key, text(of, "text"), of["ph"] == true, window, cx);
                let tall = sizes::points(of["tall"].as_f64().unwrap_or_default());
                Textarea::new(&state).h(tall).into_any_element()
            }
            "controls" => {
                let controls = self.controls(sid, "controls", &key, list(of, "ctl"), window, cx);
                h_flex().gap_2().children(controls).into_any_element()
            }
            "table" => figures::table(of, cx),
            "chart" => figures::chart(sid, index, of, cx),
            "storage" => figures::storage(of, cx),
            "cards" => self.cards(sid, of, cx),
            _ => return None,
        })
    }

    /// A page's header or a section's title, with what stands at its end.
    fn heading(
        &mut self,
        sid: &'static str,
        row: &'static str,
        key: &str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let controls = self.controls(sid, row, key, list(of, "ctl"), window, cx);
        let muted = cx.theme().muted_foreground;
        let title = div().child(text(of, "title"));
        let title = match row {
            "header" => title.text_2xl().font_semibold(),
            _ => title.text_base().font_medium(),
        };
        let under = text(of, "sub");
        h_flex()
            .w_full()
            .items_start()
            .justify_between()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(title)
                    .when(!under.is_empty(), |this| {
                        this.child(div().text_sm().text_color(muted).child(under))
                    }),
            )
            .child(h_flex().gap_2().flex_shrink_0().children(controls))
            .into_any_element()
    }

    /// A page's tabs, each a page of its own, with the page's search.
    fn tabs(
        &mut self,
        sid: &'static str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let owner = spec::owner(sid);
        let (names, counts) = (list(of, "tabs"), list(of, "counts"));
        let muted = cx.theme().muted_foreground;
        let active = of["active"].as_u64().unwrap_or(0) as usize;
        let tabs = names.iter().enumerate().map(|(index, name)| {
            let tab = Tab::new().label(name.as_str().unwrap_or(""));
            match counts.get(index).and_then(Value::as_str) {
                // The chosen tab's count takes the tab's own colour.
                Some(count) if !count.is_empty() => tab.suffix(
                    div()
                        .when(index != active, |this| this.text_color(muted))
                        .child(count),
                ),
                _ => tab,
            }
        });
        let bar = TabBar::new(SharedString::from(format!("{owner}-tabs")))
            .pill()
            .selected_index(active)
            .children(tabs)
            .on_click(cx.listener(move |this, index: &usize, _, cx| {
                let name = names.get(*index).and_then(Value::as_str).unwrap_or("");
                if let Some(page) = spec::tab(owner, name, *index) {
                    this.go(page, cx);
                }
            }));
        let find = match text(of, "search") {
            "" => None,
            hint => {
                let state = self.find(owner, hint, window, cx);
                Some(div().w(sizes::find_width()).child(Input::new(&state)))
            }
        };
        let controls = self.controls(
            sid,
            "tabs",
            &format!("{sid}|tabs"),
            list(of, "ctl"),
            window,
            cx,
        );
        h_flex()
            .w_full()
            .justify_between()
            .gap_3()
            .child(bar)
            .child(h_flex().gap_2().children(find).children(controls))
            .into_any_element()
    }

    /// The way back from a sub-page to the page it belongs to.
    fn back(&mut self, of: &'static Value, cx: &mut Context<Self>) -> AnyElement {
        let parent = text(of, "label");
        let back = Button::new("settings-back")
            .ghost()
            .small()
            .icon(marks::icon("arrow-left"))
            .label(parent)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.go(&spec::slug(parent), cx);
            }));
        h_flex().child(back).into_any_element()
    }

    /// A list with nothing in it yet, and what starts it.
    fn empty(
        &mut self,
        sid: &'static str,
        key: &str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let controls = self.controls(sid, "empty", key, list(of, "ctl"), window, cx);
        let mut header = EmptyHeader::new().title(EmptyTitle::new().child(text(of, "title")));
        if !text(of, "text").is_empty() {
            header = header.description(EmptyDescription::new().child(text(of, "text")));
        }
        let mut empty = Empty::new().header(header);
        if !controls.is_empty() {
            empty = empty.content(EmptyContent::new().children(controls));
        }
        panel(key.to_string(), cx)
            .child(div().py_6().child(empty))
            .into_any_element()
    }

    /// A list's search with the choices that narrow it.
    fn filter(
        &mut self,
        sid: &'static str,
        key: &str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut row = h_flex().w_full().gap_2();
        for (index, control) in list(of, "ctl").iter().enumerate() {
            if text(control, "c") == "input" {
                let state = self.find(sid, text(control, "text"), window, cx);
                let field = Input::new(&state).prefix(marks::icon("search"));
                row = row.child(div().flex_1().child(field));
                continue;
            }
            let key = format!("{key}|{index}");
            row = row.children(self.control(sid, "filter", key, control, window, cx));
        }
        row.into_any_element()
    }

    /// The companions to choose from: the one chosen has the ring.
    fn cards(
        &mut self,
        sid: &'static str,
        of: &'static Value,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let key = format!("{sid}|cards");
        let chosen = self.values.get(&key).and_then(Value::as_u64).unwrap_or(0) as usize;
        let theme = cx.theme();
        let (edge, ring, muted) = (theme.border, theme.primary, theme.muted_foreground);
        let (round, tile) = (theme.radius, theme.muted);
        let cards =
            list(of, "cards").iter().enumerate().map(|(index, card)| {
                let word = |at: usize| card[at].as_str().unwrap_or("");
                let key = key.clone();
                v_flex()
                    .id(SharedString::from(format!("{sid}-card-{index}")))
                    .w(sizes::card_width())
                    .items_center()
                    .gap_1()
                    .p_4()
                    .rounded(round)
                    .border_1()
                    .border_color(if index == chosen { ring } else { edge })
                    .cursor_pointer()
                    .child(
                        div()
                            .size(sizes::glyph())
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(round)
                            .bg(tile)
                            .text_xl()
                            .child(word(0)),
                    )
                    .child(div().text_sm().font_medium().child(word(1)))
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .text_center()
                            .child(word(2)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set(key.clone(), Value::from(index), cx)
                    }))
            });
        h_flex()
            .w_full()
            .flex_wrap()
            .gap_3()
            .children(cards)
            .into_any_element()
    }
}

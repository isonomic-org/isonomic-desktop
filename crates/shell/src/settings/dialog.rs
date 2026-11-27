// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The dialogs: each is a file like a page's, with a title, its blocks and
//! the buttons at its foot.

use serde_json::Value;
use ui::base::{h_flex, v_flex};
use ui::component::button::Button;
use ui::component::dialog::DialogFooter;
use ui::component::notification::Notification;
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::component::WindowExt as _;
use ui::prelude::FluentBuilder as _;
use ui::settings as sizes;
use ui::{
    div, AnyElement, Context, IntoElement, ParentElement as _, SharedString, Styled as _, Window,
};

use super::controls::styled;
use super::spec::{self, list, text};
use super::{figures, marks, Settings, NOT_YET};

impl Settings {
    /// Open the dialog called `id` in the place of any that is open.
    pub(super) fn open(&mut self, id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(of) = spec::spec(id) else {
            return;
        };
        window.close_all_dialogs(cx);
        let title = list(of, "blocks")
            .iter()
            .find(|block| text(block, "k") == "title")
            .map_or("", |block| text(block, "title"));
        let width = of["wide"].as_f64().map(sizes::points);
        let view = cx.entity();
        // The dialog is drawn again on every frame: its fields are made
        // once and kept by `Settings`.
        window.open_dialog(cx, move |dialog, window, cx| {
            let (body, foot) = view.update(cx, |this, cx| this.dialog(of, window, cx));
            dialog
                .title(title)
                .centered()
                .w(width.unwrap_or_else(sizes::dialog_width))
                .child(body)
                .footer(DialogFooter::new().children(foot))
        });
    }

    /// A dialog's blocks and the buttons at its foot.
    fn dialog(
        &mut self,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (AnyElement, Vec<AnyElement>) {
        let sid = text(of, "id");
        let (mut body, mut foot) = (v_flex().gap_4(), Vec::new());
        for (index, block) in list(of, "blocks").iter().enumerate() {
            let kind = text(block, "k");
            let key = format!("{sid}|{kind}{index}");
            let drawn = match kind {
                "title" => None,
                "actions" => {
                    foot = self.actions(sid, block, cx);
                    None
                }
                "field" => Some(self.field(sid, &key, block, window, cx)),
                "plans" => Some(self.plans(sid, &key, block, window, cx)),
                "words" => Some(figures::words(block, cx)),
                "code" => Some(figures::written(block, "code", cx)),
                "keys" => Some(figures::written(block, "keys", cx)),
                "dropzone" => Some(figures::dropzone(block, cx)),
                "log" => Some(figures::log(sid, block, cx)),
                _ => self.block(sid, index, block, "", window, cx),
            };
            body = body.children(drawn);
        }
        (body.into_any_element(), foot)
    }

    /// The buttons at a dialog's foot.
    fn actions(
        &mut self,
        sid: &'static str,
        of: &'static Value,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut buttons = Vec::new();
        for (index, action) in list(of, "ctl").iter().enumerate() {
            let label = text(action, "label");
            let id = SharedString::from(format!("{sid}-action-{index}"));
            let button = styled(Button::new(id).label(label), action);
            let pressed = cx.listener(move |this, _, window, cx| this.act(label, window, cx));
            buttons.push(button.on_click(pressed).into_any_element());
        }
        buttons
    }

    /// A button at a dialog's foot was pressed. "Cancel" and "Done" close
    /// it; any other would do something that is not connected yet, and
    /// says so.
    fn act(&mut self, label: &str, window: &mut Window, cx: &mut Context<Self>) {
        window.close_dialog(cx);
        if !matches!(label, "Cancel" | "Done") {
            window.push_notification(Notification::info(NOT_YET), cx);
        }
    }

    /// A labelled field of a dialog, with the line under it.
    fn field(
        &mut self,
        sid: &'static str,
        key: &str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let controls = self.controls(sid, "field", key, list(of, "ctl"), window, cx);
        let muted = cx.theme().muted_foreground;
        let (label, hint) = (text(of, "label"), text(of, "hint"));
        v_flex()
            .gap_1p5()
            .when(!label.is_empty(), |this| {
                this.child(div().text_sm().font_medium().child(label))
            })
            .child(h_flex().gap_2().children(controls))
            .when(!hint.is_empty(), |this| {
                this.child(div().text_xs().text_color(muted).child(hint))
            })
            .into_any_element()
    }

    /// The plans side by side, each with what it holds and its button.
    fn plans(
        &mut self,
        sid: &'static str,
        key: &str,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut row = h_flex().gap_3().items_start();
        for (index, plan) in list(of, "plans").iter().enumerate() {
            let key = format!("{key}|{index}");
            let controls = self.controls(sid, "plans", &key, list(plan, "ctl"), window, cx);
            let theme = cx.theme();
            let (edge, muted, round) = (theme.border, theme.muted_foreground, theme.radius);
            let points = list(plan, "points").iter().map(|point| {
                h_flex()
                    .gap_2()
                    .child(marks::icon("checkmark"))
                    .child(div().text_sm().child(point.as_str().unwrap_or("")))
            });
            row = row.child(
                v_flex()
                    .flex_1()
                    .gap_2()
                    .p_4()
                    .rounded(round)
                    .border_1()
                    .border_color(edge)
                    .child(div().text_sm().font_medium().child(text(plan, "name")))
                    .child(div().text_xl().font_semibold().child(text(plan, "price")))
                    .child(div().text_sm().text_color(muted).child(text(plan, "about")))
                    .children(points)
                    .children(controls),
            );
        }
        row.into_any_element()
    }
}

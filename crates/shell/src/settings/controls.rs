// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The controls at the end of a row, a heading or a field: each is the
//! library's own component, set from the page's file.

use serde_json::Value;
use ui::base::h_flex;
use ui::component::button::{
    Button, ButtonVariants as _, Toggle, ToggleGroup, ToggleVariants as _,
};
use ui::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use ui::component::kbd::Kbd;
use ui::component::menu::{DropdownMenu as _, PopupMenuItem};
use ui::component::slider::{Slider, SliderState};
use ui::component::switch::Switch;
use ui::component::tag::Tag;
use ui::component::theme::ActiveTheme as _;
use ui::prelude::FluentBuilder as _;
use ui::settings as sizes;
use ui::{
    div, tokens, AnyElement, AppContext as _, Context, Entity, IntoElement, Keystroke,
    ParentElement as _, SharedString, Styled as _, Window,
};

use super::spec::{list, text};
use super::{marks, Settings};

/// A button as the design styles it: filled, filled in red, outlined, or
/// outlined with red words.
pub(super) fn styled(button: Button, of: &'static Value) -> Button {
    let button = match (text(of, "style"), text(of, "red")) {
        ("primary", "fill") => button.danger(),
        ("primary", _) => button.primary(),
        (_, "text") => button.outline().danger(),
        _ => button.outline(),
    };
    match text(of, "icon") {
        "" => button,
        name => button.icon(marks::icon(name)),
    }
}

impl Settings {
    /// The controls of `row` on `sid`. `key` names the row among what is
    /// kept; `row` names it among the links.
    pub(super) fn controls(
        &mut self,
        sid: &'static str,
        row: &'static str,
        key: &str,
        controls: &'static [Value],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut drawn = Vec::new();
        for (index, of) in controls.iter().enumerate() {
            let key = format!("{key}|{index}");
            drawn.extend(self.control(sid, row, key, of, window, cx));
        }
        drawn
    }

    /// One control, or nothing for a part the library draws itself.
    pub(super) fn control(
        &mut self,
        sid: &'static str,
        row: &'static str,
        key: String,
        of: &'static Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let theme = cx.theme();
        let (muted, mono) = (theme.muted_foreground, theme.mono_font_family.clone());
        let (good, paused) = (theme.success, theme.warning);
        let id = SharedString::from(key.clone());
        let shown = text(of, "text");
        Some(match text(of, "c") {
            "toggle" => {
                let on = self.values.get(&key).and_then(Value::as_bool);
                let on = on.or(of["on"].as_bool()).unwrap_or(false);
                Switch::new(id)
                    .checked(on)
                    .on_click(cx.listener(move |this, on: &bool, _, cx| {
                        this.set(key.clone(), Value::Bool(*on), cx)
                    }))
                    .into_any_element()
            }
            "select" => {
                let options = list(of, "options");
                let now = self.values.get(&key).and_then(Value::as_str);
                let now = now.unwrap_or(text(of, "label")).to_string();
                let view = cx.entity();
                Button::new(id)
                    .label(now.clone())
                    .outline()
                    .dropdown_caret(true)
                    .dropdown_menu(move |mut menu, _, _| {
                        for option in options.iter().filter_map(Value::as_str) {
                            let (view, key) = (view.clone(), key.clone());
                            let item = PopupMenuItem::new(option).checked(option == now);
                            menu = menu.item(item.on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.set(key.clone(), Value::String(option.into()), cx)
                                })
                            }));
                        }
                        menu
                    })
                    .into_any_element()
            }
            "seg" => {
                let active = self.values.get(&key).and_then(Value::as_u64);
                let active = active.or(of["active"].as_u64()).unwrap_or(0) as usize;
                let options = list(of, "options").iter().filter_map(Value::as_str);
                ToggleGroup::new(id)
                    .segmented()
                    .single()
                    .outline()
                    .children(
                        options
                            .enumerate()
                            .map(|(i, option)| Toggle::new(i).label(option).checked(i == active)),
                    )
                    .on_click(cx.listener(move |this, on: &Vec<bool>, _, cx| {
                        if let Some(index) = on.iter().position(|on| *on) {
                            this.set(key.clone(), Value::from(index), cx);
                        }
                    }))
                    .into_any_element()
            }
            "button" => {
                let label = text(of, "label");
                let button = styled(Button::new(id).label(label), of);
                let menu = list(of, "menu");
                if menu.is_empty() {
                    button
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.press(sid, row, label, window, cx)
                        }))
                        .into_any_element()
                } else {
                    let view = cx.entity();
                    button
                        .dropdown_menu(move |mut popup, _, _| {
                            for entry in menu {
                                let (name, target) = (entry[0].as_str(), entry[1].as_str());
                                let (Some(name), Some(target)) = (name, target) else {
                                    continue;
                                };
                                let view = view.clone();
                                let item = PopupMenuItem::new(name);
                                popup = popup.item(item.on_click(move |_, window, cx| {
                                    view.update(cx, |this, cx| this.follow(target, window, cx))
                                }));
                            }
                            popup
                        })
                        .into_any_element()
                }
            }
            "iconbtn" => {
                let name = text(of, "name");
                let says = match name {
                    "edit" => "Edit",
                    "trash-can" => "Remove",
                    "renew" => "Refresh",
                    _ => "More",
                };
                Button::new(id)
                    .ghost()
                    .icon(marks::icon(name))
                    .tooltip(says)
                    .accessibility_label(says)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if name == "trash-can" {
                            this.remove(sid, row, cx);
                        } else {
                            this.press(sid, row, name, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            "icon" => marks::icon(text(of, "name"))
                .text_color(muted)
                .into_any_element(),
            "text" => div()
                .text_sm()
                .text_color(muted)
                .when(of["mono"] == true, |this| this.font_family(mono))
                .child(shown)
                .into_any_element(),
            "badge" => match text(of, "theme") {
                "green" => Tag::success(),
                "yellow" => Tag::warning(),
                _ => Tag::secondary(),
            }
            .child(shown)
            .into_any_element(),
            "status" => {
                let dot = match shown {
                    "Running" | "Loaded" | "Active" | "Passkey active" | "Saved" => good,
                    "Paused" => paused,
                    _ => muted,
                };
                let dot = div().size(tokens::status_dot()).rounded_full().bg(dot);
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(dot)
                    .child(div().text_sm().child(shown))
                    .into_any_element()
            }
            // The file has the keys both as the design writes them and as a
            // keystroke.
            "kbd" => match Keystroke::parse(text(of, "keys")).ok() {
                Some(keys) => Kbd::new(keys).into_any_element(),
                None => div().text_sm().child(shown).into_any_element(),
            },
            "input" => {
                let state = self.input(&key, shown, of["ph"] == true, window, cx);
                let width = of["wide"].as_f64().map(sizes::points);
                let input = Input::new(&state);
                let input = match text(of, "suffix") {
                    "" => input,
                    suffix => input.suffix(div().text_sm().text_color(muted).child(suffix)),
                };
                div()
                    .w(width.unwrap_or_else(sizes::field_width))
                    .max_w_full()
                    .child(input)
                    .into_any_element()
            }
            "textarea" => {
                let state = self.area(&key, shown, of["ph"] == true, window, cx);
                let tall = of["tall"].as_f64().unwrap_or(120.);
                div()
                    .w_full()
                    .child(Textarea::new(&state).h(sizes::points(tall)))
                    .into_any_element()
            }
            "slider" => {
                let state = self.slider(&key, shown, cx);
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(div().w(sizes::slider_width()).child(Slider::new(&state)))
                    .child(div().text_sm().text_color(muted).child(shown))
                    .into_any_element()
            }
            _ => return None,
        })
    }

    /// The field called `key`, made the first time it is drawn: holding
    /// `shown`, or showing it as a hint when `hint`.
    pub(super) fn input(
        &mut self,
        key: &str,
        shown: &'static str,
        hint: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(state) = self.inputs.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| {
            let state = InputState::new(window, cx);
            if hint {
                state.placeholder(shown)
            } else {
                state.default_value(shown)
            }
        });
        // A page's search narrows its rows as it is typed.
        cx.subscribe(&state, |_, _, _: &InputEvent, cx| cx.notify())
            .detach();
        self.inputs.insert(key.to_string(), state.clone());
        state
    }

    /// The text area called `key`, made the first time it is drawn.
    pub(super) fn area(
        &mut self,
        key: &str,
        shown: &'static str,
        hint: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextareaState> {
        if let Some(state) = self.areas.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| {
            let state = TextareaState::new(window, cx);
            if hint {
                state.placeholder(shown)
            } else {
                state.default_value(shown)
            }
        });
        self.areas.insert(key.to_string(), state.clone());
        state
    }

    fn slider(&mut self, key: &str, shown: &str, cx: &mut Context<Self>) -> Entity<SliderState> {
        if let Some(state) = self.sliders.get(key) {
            return state.clone();
        }
        let at = shown.parse::<f32>().unwrap_or(50.);
        let state = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(at));
        self.sliders.insert(key.to_string(), state.clone());
        state
    }
}

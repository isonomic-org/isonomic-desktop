// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The list rows onboarding and Settings share: a mark, a name with its
//! line, what is said beside it, and its control. A model's row has its
//! Recommended or Not recommended pill, whose "i" says why when pointed
//! at; a connector's says where its connection stands.
//!
//! Every size and colour is onboarding's (`ui::onboarding`).

use std::sync::Arc;

use isonomic_proto::connections::{Connection, ConnectionState};
use ui::assets::IconName;
use ui::component::tag::Tag;
use ui::component::tooltip::Tooltip;
use ui::component::Icon;
use ui::onboarding::passkey::{self as fourth};
use ui::onboarding::steps::{self as design};
use ui::onboarding::{self as page};
use ui::{
    div, img, px, AnyElement, Div, ElementId, FontWeight, Image, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
};

use crate::workspace::Model;

pub fn text(size: (f32, f32)) -> Div {
    div().text_size(px(size.0)).line_height(px(size.1))
}

/// A name with a line about it under it.
pub fn named(name: impl Into<SharedString>, about: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            text(fourth::ROW_TITLE)
                .font_weight(FontWeight::MEDIUM)
                .text_color(page::ink())
                .child(name.into()),
        )
        .child(
            text(design::SMALL)
                .text_color(fourth::faint())
                .child(about.into()),
        )
}

/// A row: its mark, its name and line, what is beside them, and its
/// control, over a hairline.
pub fn row(picture: AnyElement, words: Div, beside: Vec<AnyElement>, control: AnyElement) -> Div {
    div()
        .w_full()
        .h(px(design::ROW))
        .flex()
        .items_center()
        .gap(px(design::LOGO_GAP))
        .border_b_1()
        .border_color(page::hairline())
        .child(picture)
        .child(words)
        .children(beside)
        .child(control)
}

/// A mark: the logo, or the name's first letter on a chip where there is
/// none.
pub fn mark(logo: Option<Arc<Image>>, name: &str) -> AnyElement {
    match logo {
        Some(logo) => img(logo)
            .flex_none()
            .size(px(design::LOGO))
            .into_any_element(),
        None => div()
            .flex_none()
            .size(px(design::LOGO))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(fourth::CARD_CORNER))
            .border_1()
            .border_color(page::hairline())
            .bg(design::field())
            .text_color(page::ink())
            .font_weight(FontWeight::SEMIBOLD)
            .child(monogram(name))
            .into_any_element(),
    }
}

fn monogram(name: &str) -> String {
    name.chars()
        .find(|letter| letter.is_alphanumeric())
        .map_or_else(String::new, |letter| letter.to_uppercase().collect())
}

/// A model's pill: Recommended or Not recommended, with the "i" whose
/// tooltip says why, for this Mac.
pub fn fit_pill(id: impl Into<ElementId>, model: &Model) -> AnyElement {
    let tag = if model.fits {
        Tag::custom(design::good(), design::good_ink(), design::good())
    } else {
        Tag::custom(design::warning(), design::warning_ink(), design::warning())
    };
    let reason: SharedString = model.reason().into();
    let info = div()
        .id(id)
        .flex_none()
        .child(Icon::from(IconName::Information).size(px(design::PILL_ICON)))
        .tooltip(move |window, cx| Tooltip::new(reason.clone()).build(window, cx));
    div()
        .flex_none()
        .child(
            tag.py_0p5()
                .gap_1()
                .items_center()
                .child(model.pill())
                .child(info),
        )
        .into_any_element()
}

/// "Coming soon", where a connector's switch would be.
pub fn coming_soon() -> AnyElement {
    div()
        .flex_none()
        .child(Tag::secondary().outline().py_0p5().child("Coming soon"))
        .into_any_element()
}

/// Where a connection stands, in a few words.
pub fn standing(connection: &Connection) -> String {
    match connection.state {
        ConnectionState::Ready if connection.tools > 0 => {
            format!("Connected \u{b7} {} tools", connection.tools)
        }
        ConnectionState::Ready => "Connected".to_owned(),
        ConnectionState::NeedsSignIn => "Sign in to finish connecting".to_owned(),
        ConnectionState::Connecting => "Connecting\u{2026}".to_owned(),
        ConnectionState::Failed => connection
            .error
            .clone()
            .unwrap_or_else(|| "It couldn\u{2019}t connect.".to_owned()),
        _ => "Its state is one this window doesn\u{2019}t know.".to_owned(),
    }
}

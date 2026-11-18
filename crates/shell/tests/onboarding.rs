// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The onboarding window's view in a headless window: once its motion has
//! ended, the title and both buttons are in place, and a button tells the
//! window what was chosen.

extern crate ui as gpui;

use std::cell::RefCell;
use std::rc::Rc;

use shell::onboarding::{self, Chosen, Onboarding};
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Onboarding>) {
    cx.update(|cx| {
        ui::init(cx);
        let options = onboarding::window_options(cx);
        ui::open_window(options, cx, |_, cx| cx.new(Onboarding::new)).expect("open test window")
    })
}

fn shown(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window
            .try_find(id)
            .map(|element| element.visible())
            .unwrap_or(false)
    })
    .unwrap()
}

#[ui::test]
fn the_window_is_the_design_s_fixed_page(cx: &mut TestAppContext) {
    let options = cx.update(|cx| onboarding::window_options(cx));
    assert!(options.titlebar.is_none());
    assert!(!options.is_resizable && !options.is_minimizable);
}

#[ui::test]
fn once_settled_it_offers_the_two_ways_to_start(cx: &mut TestAppContext) {
    let (window, view) = open(cx);
    assert!(shown(cx, window, "onboarding"));
    view.update(cx, |view, cx| view.skip(cx));
    assert!(view.read_with(cx, |view, _| view.settled()));
    for id in [
        "onboarding-title",
        "onboarding-new-account",
        "onboarding-recover",
    ] {
        assert!(shown(cx, window, id), "{id}");
    }
}

#[ui::test]
fn a_button_says_what_was_chosen(cx: &mut TestAppContext) {
    let (window, view) = open(cx);
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let heard = Rc::clone(&chosen);
    cx.update(|cx| {
        cx.subscribe(&view, move |_, choice: &Chosen, _| {
            heard.borrow_mut().push(*choice)
        })
        .detach();
    });
    view.update(cx, |view, cx| view.skip(cx));
    for id in ["onboarding-new-account", "onboarding-recover"] {
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
    }
    cx.run_until_parked();
    assert_eq!(*chosen.borrow(), [Chosen::NewAccount, Chosen::Recover]);
}

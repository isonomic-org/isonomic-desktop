// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The onboarding window's view in a headless window: once its motion has
//! ended, the title and both buttons are in place; a new account goes
//! through the third screen, whose buttons decline or accept.

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
    let click = |cx: &mut TestAppContext, id: &'static str| {
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    click(cx, "onboarding-recover");
    assert_eq!(*chosen.borrow(), [Chosen::Recover]);

    // A new account goes to the third screen first, and is only chosen
    // once its terms are accepted.
    click(cx, "onboarding-new-account");
    assert_eq!(*chosen.borrow(), [Chosen::Recover]);
    view.update(cx, |view, cx| view.skip_terms(cx));
    for id in ["onboarding-terms", "onboarding-stepper"] {
        assert!(shown(cx, window, id), "{id}");
    }

    // Declining goes back to the two ways to start.
    click(cx, "onboarding-decline");
    view.update(cx, |view, cx| view.skip_rewind(cx));
    assert!(view.read_with(cx, |view, _| view.terms_now().is_none()));
    click(cx, "onboarding-new-account");
    view.update(cx, |view, cx| view.skip_terms(cx));
    // Accepting plays the signing first, then the fourth screen comes on.
    click(cx, "onboarding-accept");
    assert_eq!(*chosen.borrow(), [Chosen::Recover]);
    click(cx, "onboarding-decline");
    assert!(view.read_with(cx, |view, _| view.terms_now().is_some()));
    view.update(cx, |view, cx| view.skip_signing(cx));
    view.update(cx, |view, cx| view.skip_passkey(cx));
    cx.run_until_parked();
    assert!(shown(cx, window, "onboarding-passkey"));
    assert_eq!(*chosen.borrow(), [Chosen::Recover]);

    // "Previous step" goes back to the terms, which can be accepted again.
    click(cx, "onboarding-previous");
    view.update(cx, |view, cx| view.skip_rewind(cx));
    assert!(!shown(cx, window, "onboarding-passkey"));
    click(cx, "onboarding-accept");
    view.update(cx, |view, cx| view.skip_signing(cx));
    view.update(cx, |view, cx| view.skip_passkey(cx));
    cx.run_until_parked();

    // "Continue with Face ID" brings up the sheet, then the later steps,
    // each over the last; the account is chosen after the last, once.
    // "Cancel" on the sheet goes back to the screen, and it can be asked
    // for again.
    click(cx, "onboarding-face-id");
    assert!(shown(cx, window, "onboarding-face-id-sheet"));
    click(cx, "onboarding-cancel");
    assert!(!shown(cx, window, "onboarding-face-id-sheet"));
    assert!(shown(cx, window, "onboarding-passkey"));
    click(cx, "onboarding-face-id");
    assert!(shown(cx, window, "onboarding-face-id-sheet"));
    view.update(cx, |view, cx| view.skip_flow(cx));
    cx.run_until_parked();
    for step in [
        "onboarding-created",
        "onboarding-kit",
        "onboarding-confirm",
        "onboarding-models",
        "onboarding-links",
        "onboarding-finish",
    ] {
        assert!(shown(cx, window, step), "{step}");
        assert_eq!(*chosen.borrow(), [Chosen::Recover]);
        if step == "onboarding-kit" {
            // "Previous step" goes back, and the step comes on again.
            click(cx, "onboarding-back");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, "onboarding-created") && !shown(cx, window, step));
            click(cx, "onboarding-next");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, step));
        }
        if step == "onboarding-kit" {
            // Not past it while the kit isn't ticked as stored.
            view.update(cx, |view, cx| view.store_kit(false, cx));
            click(cx, "onboarding-next");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, step) && !shown(cx, window, "onboarding-confirm"));
            view.update(cx, |view, cx| view.store_kit(true, cx));
            // Nor until its file is saved: the words alone restore nothing.
            click(cx, "onboarding-next");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, step) && !shown(cx, window, "onboarding-confirm"));
            let file =
                std::env::temp_dir().join(format!("onboarding-kit-{}.json", std::process::id()));
            view.update(cx, |view, cx| view.save_kit_to(file.clone(), cx));
            cx.run_until_parked();
            let saved = std::fs::read_to_string(&file).expect("the kit file");
            let _ = std::fs::remove_file(&file);
            assert!(saved.contains(onboarding::backend::SAMPLE_ID) && !saved.contains("hollow"));
        }
        if step == "onboarding-confirm" {
            // Not past it until the kit's three words are typed back.
            click(cx, "onboarding-next");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, step));
            // Each is typed in its own input: a press on it, then the word.
            // Capital letters and the spaces around a word don't matter; a
            // wrong word does, until it is put right.
            for (index, word) in [" Hollow", "RIBBON ", "sketches"].into_iter().enumerate() {
                cx.update_window(window, |_, window, cx| {
                    window.render_frame(cx);
                    window.click(("onboarding-word", index), cx);
                })
                .unwrap();
                cx.run_until_parked();
                cx.simulate_input(window, word);
                cx.run_until_parked();
            }
            click(cx, "onboarding-next");
            view.update(cx, |view, cx| view.skip_flow(cx));
            assert!(shown(cx, window, step) && !shown(cx, window, "onboarding-models"));
            cx.simulate_keystrokes(window, "backspace backspace");
            cx.run_until_parked();
        }
        click(cx, "onboarding-next");
        view.update(cx, |view, cx| view.skip_flow(cx));
    }
    view.update(cx, |view, cx| view.skip_flow(cx));
    cx.run_until_parked();
    assert_eq!(*chosen.borrow(), [Chosen::Recover, Chosen::NewAccount]);
}

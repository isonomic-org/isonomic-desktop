// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding over a backend that can refuse: a refusal is said under the
//! step's buttons and the step can be tried again; the kit's file holds
//! no word; and "Finish setup" hands over what was chosen and fetches the
//! models switched on.

extern crate ui as gpui;

use std::sync::Arc;

use shell::onboarding::backend::{Memory, Sample, SAMPLE_ID};
use shell::onboarding::{self, Onboarding};
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, ElementId, Entity, TestAppContext};

const REFUSED: &str = "The key store did not let Isonomic save your ID. Unlock it and try again.";

struct Run {
    window: AnyWindowHandle,
    view: Entity<Onboarding>,
    sample: Arc<Sample>,
}

impl Run {
    fn open(cx: &mut TestAppContext) -> Run {
        let sample = Arc::new(Sample::default());
        let backend = Arc::clone(&sample);
        let (window, view) = cx.update(|cx| {
            ui::init(cx);
            let options = onboarding::window_options(cx);
            ui::open_window(options, cx, |_, cx| {
                cx.new(|cx| Onboarding::over(backend, cx))
            })
            .expect("open test window")
        });
        Run {
            window,
            view,
            sample,
        }
    }

    fn click(&self, cx: &mut TestAppContext, id: impl Into<ElementId>) {
        let id = id.into();
        cx.update_window(self.window, |_, window, cx| {
            window.render_frame(cx);
            window.click(id, cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    fn shown(&self, cx: &mut TestAppContext, id: &'static str) -> bool {
        cx.update_window(self.window, |_, window, cx| {
            window.render_frame(cx);
            window.try_find(id).is_some_and(|element| element.visible())
        })
        .unwrap()
    }

    fn skip(&self, cx: &mut TestAppContext) {
        self.view.update(cx, |view, cx| view.skip_flow(cx));
        cx.run_until_parked();
    }

    /// To "Create your account with Face ID".
    fn to_face_id(&self, cx: &mut TestAppContext) {
        self.view.update(cx, |view, cx| view.skip(cx));
        self.click(cx, "onboarding-new-account");
        self.view.update(cx, |view, cx| view.skip_terms(cx));
        self.click(cx, "onboarding-accept");
        self.view.update(cx, |view, cx| view.skip_signing(cx));
        self.view.update(cx, |view, cx| view.skip_passkey(cx));
        cx.run_until_parked();
    }

    /// From there to "Save your recovery kit".
    fn to_kit(&self, cx: &mut TestAppContext) {
        self.click(cx, "onboarding-face-id");
        self.skip(cx);
        self.click(cx, "onboarding-next");
        self.skip(cx);
    }

    /// From there to "Turn on local AI models".
    fn to_models(&self, cx: &mut TestAppContext) {
        let file = std::env::temp_dir().join(format!("onboarding-kit-{}.json", std::process::id()));
        self.view.update(cx, |view, cx| {
            view.store_kit(true, cx);
            view.save_kit_to(file.clone(), cx);
        });
        // The file is written on a thread of its own.
        for _ in 0..500 {
            cx.run_until_parked();
            if self.view.read_with(cx, |view, _| view.kit_file().is_some()) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let _ = std::fs::remove_file(file);
        self.click(cx, "onboarding-next");
        self.skip(cx);
        self.view.update(cx, |view, cx| view.fill_words(cx));
        self.click(cx, "onboarding-next");
        self.skip(cx);
    }
}

#[ui::test]
fn accepting_keeps_the_terms_and_face_id_makes_the_id(cx: &mut TestAppContext) {
    let run = Run::open(cx);
    run.to_face_id(cx);
    assert!(run.sample.calls().contains(&"accept_terms".to_owned()));
    assert_eq!(run.view.read_with(cx, |view, _| view.id()), None);
    run.click(cx, "onboarding-face-id");
    run.skip(cx);
    assert!(run.shown(cx, "onboarding-created"));
    assert_eq!(
        run.view.read_with(cx, |view, _| view.id()).as_deref(),
        Some(SAMPLE_ID)
    );
}

#[ui::test]
fn a_refused_id_is_said_and_can_be_tried_again(cx: &mut TestAppContext) {
    let run = Run::open(cx);
    run.to_face_id(cx);
    run.sample.refuse("create_account", REFUSED);
    run.click(cx, "onboarding-face-id");
    // The sheet goes; the screen stays, with what the engine said.
    assert!(!run.shown(cx, "onboarding-face-id-sheet") && run.shown(cx, "onboarding-passkey"));
    assert_eq!(
        run.view.read_with(cx, |view, _| view.trouble()).as_deref(),
        Some(REFUSED)
    );
    run.skip(cx);
    assert!(!run.shown(cx, "onboarding-created"));

    run.sample.recover();
    run.click(cx, "onboarding-face-id");
    assert_eq!(run.view.read_with(cx, |view, _| view.trouble()), None);
    run.skip(cx);
    assert!(run.shown(cx, "onboarding-created"));
}

#[ui::test]
fn a_kit_that_does_not_come_holds_the_step_until_it_does(cx: &mut TestAppContext) {
    let run = Run::open(cx);
    run.to_face_id(cx);
    run.sample.refuse("recovery_kit", REFUSED);
    run.to_kit(cx);
    assert!(run.shown(cx, "onboarding-kit"));
    assert_eq!(
        run.view.read_with(cx, |view, _| view.trouble()).as_deref(),
        Some(REFUSED)
    );
    // Ticked or not, there is nothing to go on with.
    run.view.update(cx, |view, cx| view.store_kit(true, cx));
    run.click(cx, "onboarding-next");
    run.skip(cx);
    assert!(run.shown(cx, "onboarding-kit") && !run.shown(cx, "onboarding-confirm"));

    run.sample.recover();
    run.view.update(cx, |view, cx| view.fetch_kit(cx));
    cx.run_until_parked();
    assert_eq!(run.view.read_with(cx, |view, _| view.trouble()), None);
    // The engine is told the kit is saved only once its words are typed
    // back, and a refusal there holds the step too.
    run.sample.refuse("kit_saved", REFUSED);
    run.to_models(cx);
    assert!(run.shown(cx, "onboarding-confirm") && !run.shown(cx, "onboarding-models"));
    assert_eq!(
        run.view.read_with(cx, |view, _| view.trouble()).as_deref(),
        Some(REFUSED)
    );
    run.sample.recover();
    run.click(cx, "onboarding-next");
    run.skip(cx);
    assert!(run.shown(cx, "onboarding-models"));
    assert!(run.sample.calls().contains(&"kit_saved".to_owned()));
}

#[ui::test]
fn finishing_keeps_the_choices_and_fetches_the_models_switched_on(cx: &mut TestAppContext) {
    let run = Run::open(cx);
    run.to_face_id(cx);
    run.to_kit(cx);
    run.to_models(cx);
    // The design's sample has its first and sixth models on: the second
    // is switched on too.
    run.click(cx, ("onboarding-switch", 3 * 16 + 1usize));
    run.click(cx, "onboarding-next");
    run.skip(cx);
    // Nothing is connected until asked for.
    run.click(cx, ("onboarding-connect", 2usize));
    run.click(cx, "onboarding-next");
    run.skip(cx);
    assert!(run.shown(cx, "onboarding-finish"));
    run.click(cx, "onboarding-next");
    cx.run_until_parked();

    let choices = run.sample.choices().expect("the choices are kept");
    assert_eq!(
        choices.models,
        ["qwen3.5-4b", "mistral-small-4", "qwen3.5-0.8b"]
    );
    assert_eq!(choices.integrations, ["GitHub"]);
    assert!(!choices.web_access && choices.snapshots);
    let calls = run.sample.calls();
    for model in &choices.models {
        assert!(calls.contains(&format!("download {model}")), "{model}");
    }
}

#[ui::test]
fn a_mac_short_of_memory_fetches_no_model(cx: &mut TestAppContext) {
    let run = Run::open(cx);
    run.sample.with_memory(Memory::Short {
        has: 8 << 30,
        needs: 16 << 30,
    });
    run.to_face_id(cx);
    run.to_kit(cx);
    run.to_models(cx);
    assert!(run.shown(cx, "onboarding-models"));
    // No switch is on, and none can be put on.
    run.click(cx, ("onboarding-switch", 3 * 16usize));
    assert!(run
        .view
        .read_with(cx, |view, _| view.choices().models.is_empty()));
    for _ in 0..3 {
        run.click(cx, "onboarding-next");
        run.skip(cx);
    }
    cx.run_until_parked();
    assert!(!run
        .sample
        .calls()
        .iter()
        .any(|call| call.starts_with("download")));
}

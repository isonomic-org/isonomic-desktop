// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures of onboarding's screens, each at rest, for checking them
//! against the design: the real view in a headless window with a real
//! renderer, saved as `NN-name.png`.
//!
//! It runs only when `ONBOARDING_SHOTS` names the folder to save them in,
//! and is skipped where no headless renderer starts:
//!
//! ```sh
//! ONBOARDING_SHOTS=/tmp/shots cargo test -p shell --test onboarding_shots
//! ```
//!
//! What moves by a timer of its own (the laptop, the clipboard, the pen
//! and the seal) may be missing from a picture; the panels never are.

extern crate ui as gpui;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use shell::onboarding::backend::{Memory, Sample};
use shell::onboarding::{self, Chosen, Onboarding};
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, Entity, HeadlessAppContext};

struct Shots {
    cx: HeadlessAppContext,
    window: AnyWindowHandle,
    view: Entity<Onboarding>,
    sample: Arc<Sample>,
    folder: PathBuf,
    taken: usize,
}

impl Shots {
    fn open(folder: PathBuf) -> Option<Shots> {
        // No GPU and no software driver: no renderer starts.
        let mut cx = std::panic::catch_unwind(|| {
            let platform = ui::platform::current_platform(true);
            HeadlessAppContext::with_platform(
                platform.text_system(),
                std::sync::Arc::new(ui::assets::AllAssets),
                ui::platform::current_headless_renderer,
            )
        })
        .ok()?;
        cx.update(ui::init);
        let sample = Arc::new(Sample::default());
        let backend = Arc::clone(&sample);
        let (window, view) = cx
            .update(|cx| {
                let options = ui::WindowOptions {
                    focus: false,
                    show: false,
                    ..onboarding::window_options(cx)
                };
                ui::open_window(options, cx, |_, cx| {
                    cx.new(|cx| Onboarding::over(backend, cx))
                })
            })
            .ok()?;
        cx.capture_screenshot(window).ok()?;
        Some(Shots {
            cx,
            window,
            view,
            sample,
            folder,
            taken: 0,
        })
    }

    fn act(&mut self, act: impl FnOnce(&mut Onboarding, &mut ui::Context<Onboarding>)) {
        let view = self.view.clone();
        self.cx.update(|cx| view.update(cx, act));
        self.cx.run_until_parked();
    }

    /// Draw until what is drawn off the foreground thread has come, then
    /// save the window as it is.
    fn shot(&mut self, name: &str) {
        self.shot_after(12, name);
    }

    /// The same after `rounds` draws, each 40 ms after the last.
    fn shot_after(&mut self, rounds: usize, name: &str) {
        for _ in 0..rounds {
            // Timers run by this clock, not the wall's.
            self.cx.advance_clock(Duration::from_millis(40));
            self.cx.run_until_parked();
            let _ = self
                .cx
                .update_window(self.window, |_, window, cx| window.render_frame(cx));
            std::thread::sleep(Duration::from_millis(40));
        }
        let image = self
            .cx
            .update_window(self.window, |_, window, cx| {
                window.render_frame(cx);
                window.render_to_image()
            })
            .expect("the window is open")
            .expect("the window renders");
        self.taken += 1;
        let path = self.folder.join(format!("{:02}-{name}.png", self.taken));
        image.save(&path).expect("the picture saves");
    }
}

impl Shots {
    /// Tick the kit as stored and save its file, as its step asks.
    fn keep_kit(&mut self) {
        let file =
            std::env::temp_dir().join(format!("onboarding-shots-kit-{}.json", std::process::id()));
        let to = file.clone();
        self.act(|view, cx| {
            view.store_kit(true, cx);
            view.save_kit_to(to, cx);
        });
        let _ = std::fs::remove_file(file);
    }

    /// `ONBOARDING_SHOTS_TROUBLE` is set: pictures of what is shown when
    /// the engine refuses, or this Mac is short of memory, instead of the
    /// screens at rest.
    fn troubles(mut self) {
        const SAYS: &str =
            "The key store did not let Isonomic save your ID. Unlock it and try again.";
        self.sample.with_memory(Memory::Short {
            has: 8 << 30,
            needs: 16 << 30,
        });
        self.act(|view, cx| view.skip(cx));
        self.act(|view, cx| view.choose(Chosen::NewAccount, cx));
        self.act(|view, cx| view.skip_terms(cx));
        self.act(|view, cx| view.accept(cx));
        self.act(|view, cx| view.skip_signing(cx));
        self.act(|view, cx| view.skip_passkey(cx));
        self.sample.refuse("create_account", SAYS);
        self.act(|view, cx| view.continue_with_face_id(cx));
        self.shot("id-refused");
        self.sample.recover();
        self.sample.refuse("recovery_kit", SAYS);
        self.act(|view, cx| view.continue_with_face_id(cx));
        self.act(|view, cx| view.skip_flow(cx));
        self.act(|view, cx| view.next_step(cx));
        self.act(|view, cx| view.skip_flow(cx));
        self.shot("kit-refused");
        self.sample.recover();
        self.act(|view, cx| view.fetch_kit(cx));
        self.shot("kit-unsaved");
        self.keep_kit();
        self.shot("kit-saved");
        self.act(|view, cx| view.next_step(cx));
        self.act(|view, cx| view.skip_flow(cx));
        self.act(|view, cx| view.fill_words(cx));
        self.sample.refuse("kit_saved", SAYS);
        self.act(|view, cx| view.next_step(cx));
        self.shot("kit-not-kept");
        self.sample.recover();
        self.act(|view, cx| view.next_step(cx));
        self.act(|view, cx| view.skip_flow(cx));
        self.shot("models-short-of-memory");
    }

    /// `ONBOARDING_SHOTS_BACK` is set: pictures of "Decline" and of
    /// "Previous step" on the fourth screen under way, about two a second,
    /// instead of the screens at rest.
    fn going_back(mut self) {
        self.act(|view, cx| view.skip(cx));
        self.act(|view, cx| view.choose(Chosen::NewAccount, cx));
        self.act(|view, cx| view.skip_terms(cx));
        self.shot("terms");
        self.act(|view, cx| view.decline(cx));
        for _ in 0..24 {
            self.shot_after(1, "declining");
        }
        self.act(|view, cx| view.choose(Chosen::NewAccount, cx));
        self.act(|view, cx| view.skip_terms(cx));
        self.act(|view, cx| view.accept(cx));
        // The signing, then the fourth screen coming on.
        for _ in 0..18 {
            self.shot("signing");
        }
        self.act(|view, cx| view.previous_step(cx));
        for _ in 0..40 {
            self.shot_after(1, "going-back");
        }
    }
}

/// Not under the test harness: the Mac's platform starts only on the main
/// thread.
fn main() {
    let Some(folder) = std::env::var_os("ONBOARDING_SHOTS").map(PathBuf::from) else {
        eprintln!("skipped: ONBOARDING_SHOTS names no folder");
        return;
    };
    std::fs::create_dir_all(&folder).expect("the folder can be made");
    let Some(mut shots) = Shots::open(folder) else {
        eprintln!("skipped: no headless renderer here");
        return;
    };
    if std::env::var_os("ONBOARDING_SHOTS_BACK").is_some() {
        return shots.going_back();
    }
    if std::env::var_os("ONBOARDING_SHOTS_TROUBLE").is_some() {
        return shots.troubles();
    }
    shots.act(|view, cx| view.skip(cx));
    shots.shot("get-started");
    shots.act(|view, cx| view.choose(Chosen::NewAccount, cx));
    shots.act(|view, cx| view.skip_terms(cx));
    shots.shot("terms");
    shots.act(|view, cx| view.accept(cx));
    shots.act(|view, cx| view.skip_signing(cx));
    shots.act(|view, cx| view.skip_passkey(cx));
    shots.shot("passkey");
    shots.act(|view, cx| view.continue_with_face_id(cx));
    std::thread::sleep(Duration::from_millis(400));
    shots.shot("face-id-sheet");
    for step in ["created", "kit", "confirm"] {
        shots.act(|view, cx| view.skip_flow(cx));
        if step == "kit" {
            shots.keep_kit();
        }
        shots.shot(step);
        shots.act(|view, cx| view.next_step(cx));
    }
    // The last press found no words typed.
    shots.shot("confirm-wrong");
    shots.act(|view, cx| view.fill_words(cx));
    shots.act(|view, cx| view.next_step(cx));
    shots.shot("confirm-right");
    for step in ["models", "links", "finish"] {
        shots.act(|view, cx| view.skip_flow(cx));
        shots.shot(step);
        shots.act(|view, cx| view.next_step(cx));
    }
    std::thread::sleep(Duration::from_millis(1500));
    shots.shot("setting-up");
}

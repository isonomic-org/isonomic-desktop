// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding: its own window, apart from the app's (design "Onboarding
//! 0.4"). Fixed at 1280 × 800, all page, with no title bar and nothing to
//! close, shrink or grow it by. It opens before the app's window does, and
//! hands over when the person has chosen how to start.
//!
//! So far it holds the first two screens as one continuous motion: the
//! mark plays its intro in the centre with its sound, turns and shrinks
//! into its corner,
//! the laptop somersaults up from the bottom edge as its display lights,
//! and the title and the two buttons come up after it.
//!
//! Pressing "Start new account" brings the third screen, "Before you
//! continue" (`terms`), which runs from its own clock.
//!
//! Everything is drawn from one clock that starts at the first frame. The
//! laptop's frames are drawn off the foreground thread, each for the time
//! it will be shown at.

mod laptop;
mod props;
mod signature;
mod signing;
mod signing_view;
mod sound;
mod stage;
mod terms;
mod terms_view;
mod view;

use std::sync::Arc;
use std::time::{Duration, Instant};

use ui::onboarding::{self as design, at};
use ui::{
    App, AppContext as _, Bounds, Context, Entity, EventEmitter, Image, WindowBackgroundAppearance,
    WindowBounds, WindowOptions,
};

use laptop::Scene;
use props::Props;
use sound::Sound;
use stage::Drawn;

/// The shortest time between two of the laptop's frames.
const FRAME: Duration = Duration::from_millis(16);

/// What the person chose: to recover an account, on "Get started", or a
/// new one, once they have accepted the terms on the third screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    NewAccount,
    Recover,
}

pub struct Onboarding {
    /// When the first frame was drawn: every motion is timed from it.
    started: Option<Instant>,
    /// Pixels to a point on the window's screen, as of the last draw.
    scale: f32,
    /// The laptop's model, once read.
    scene: Option<Arc<Scene>>,
    /// The turning laptop's latest frame.
    frame: Option<Drawn>,
    /// The picture the laptop lands on.
    picture: Arc<Image>,
    /// The mark's sound, in the window a person sees; none in a test's.
    sound: Option<Sound>,
    /// When "Start new account" was pressed: the third screen's clock.
    terms: Option<Instant>,
    /// The clipboard's and the pen's models, once read.
    props: Option<Arc<Props>>,
    /// Their latest frames. They rest as the last ones drawn.
    clipboard: Option<Drawn>,
    pen: Option<Drawn>,
    /// When "Accept and continue" was pressed, on the third screen's
    /// clock, and whether onboarding has handed over since.
    signed: Option<f32>,
    handed_over: bool,
}

impl EventEmitter<Chosen> for Onboarding {}

impl Onboarding {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Read the model while the mark plays.
        cx.spawn(async move |view, cx| {
            let scene = cx.background_spawn(async { Scene::load() }).await;
            let _ = view.update(cx, |view, _| view.scene = scene.ok().map(Arc::new));
            let props = cx.background_spawn(async { Props::load() }).await;
            let _ = view.update(cx, |view, _| view.props = props.ok().map(Arc::new));
        })
        .detach();
        Self {
            started: None,
            scale: 1.,
            scene: None,
            frame: None,
            picture: laptop::picture(),
            sound: None,
            terms: None,
            props: None,
            clipboard: None,
            pen: None,
            signed: None,
            handed_over: false,
        }
    }

    /// Play the mark's sound with its intro.
    pub fn with_sound(mut self) -> Self {
        self.sound = Sound::load();
        self
    }

    /// Seconds since the first frame; zero before it.
    pub fn now(&self) -> f32 {
        self.started
            .map_or(0., |started| started.elapsed().as_secs_f32())
    }

    /// Jump to the end of the motion, as a test or a returning person would
    /// want it.
    pub fn skip(&mut self, cx: &mut Context<Self>) {
        let end = Duration::from_secs_f32(at::END + 1.);
        self.started = Some(Instant::now().checked_sub(end).unwrap_or_else(Instant::now));
        cx.notify();
    }

    /// Has every motion ended?
    pub fn settled(&self) -> bool {
        self.now() >= at::END
    }

    /// A press on one of the two ways to start. A new account goes on to
    /// the third screen; recovering one is left to whoever listens.
    pub fn choose(&mut self, choice: Chosen, cx: &mut Context<Self>) {
        match choice {
            Chosen::NewAccount => self.begin_terms(cx),
            Chosen::Recover => cx.emit(choice),
        }
    }

    /// The sound belongs to the mark's intro alone: fade it out so it is
    /// silent as the mark leaves the centre, then stop it.
    fn fade_sound(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |view, cx| {
            let fades_at = at::MARK_LEAVES - at::SOUND_FADE;
            cx.background_executor()
                .timer(Duration::from_secs_f32(fades_at))
                .await;
            loop {
                let Ok(left) = view.update(cx, |view, _| {
                    let left = 1. - (view.now() - fades_at) / at::SOUND_FADE;
                    if let Some(sound) = &view.sound {
                        if left > 0. {
                            sound.set_level(left);
                        } else {
                            sound.stop();
                        }
                    }
                    left
                }) else {
                    return;
                };
                if left <= 0. {
                    return;
                }
                cx.background_executor().timer(FRAME).await;
            }
        })
        .detach();
    }

    /// Start the clock and the laptop's frames. Called at the first draw.
    fn start(&mut self, cx: &mut Context<Self>) {
        self.started = Some(Instant::now());
        if let Some(sound) = &self.sound {
            sound.play();
            self.fade_sound(cx);
        }
        cx.spawn(async move |view, cx| {
            // The first frame is drawn a little ahead of the flight.
            let lead = at::LAPTOP_RISES - 0.25;
            cx.background_executor()
                .timer(Duration::from_secs_f32(lead))
                .await;
            let flight_ends = at::LAPTOP_RISES + at::LAPTOP_FLIGHT;
            let mut drawn = 0;
            // How long a frame takes to draw: it is drawn for that far ahead.
            let mut lead = 1. / 60.;
            loop {
                let Ok((now, scale, scene)) =
                    view.update(cx, |view, _| (view.now(), view.scale, view.scene.clone()))
                else {
                    return;
                };
                let Some(scene) = scene else {
                    // No model: the picture alone fades in at the end.
                    return;
                };
                let began = Instant::now();
                let last = now >= flight_ends;
                // Drawn for when it will be on screen.
                let shown_at = (now + lead).min(flight_ends);
                let image = cx
                    .background_spawn(async move { laptop::draw(&scene, shown_at, scale) })
                    .await;
                lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                let kept = view.update(cx, |view, cx| {
                    let frame = Drawn {
                        image,
                        at: shown_at,
                    };
                    if let Some(old) = view.frame.replace(frame) {
                        cx.drop_image(old.image, None);
                    }
                    cx.notify();
                });
                drawn += 1;
                if kept.is_err() || last {
                    break;
                }
                // No faster than the screen shows them.
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
            reports::log::line(&format!(
                "onboarding: the laptop's flight drew {drawn} frames in {:.1} s",
                at::LAPTOP_FLIGHT + 0.25
            ));
            // Once the picture has taken over, the last frame goes.
            cx.background_executor()
                .timer(Duration::from_secs_f32(at::SETTLE + 0.1))
                .await;
            let _ = view.update(cx, |view, cx| {
                if let Some(old) = view.frame.take() {
                    cx.drop_image(old.image, None);
                }
                cx.notify();
            });
        })
        .detach();
    }
}

/// How the onboarding window opens: centred, fixed, with no title bar. Its
/// corners are the page's own, so the window behind them is clear.
pub fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            design::window_size(),
            cx,
        ))),
        titlebar: None,
        is_resizable: false,
        is_minimizable: false,
        is_movable: true,
        window_background: WindowBackgroundAppearance::Transparent,
        ..Default::default()
    }
}

/// Open onboarding. When the person has accepted the terms for a new
/// account the window goes and `next` runs, which opens the app's window.
pub fn open(cx: &mut App, next: impl FnOnce(&mut App) + 'static) -> Option<Entity<Onboarding>> {
    let mut next = Some(next);
    let opened = ui::open_window(window_options(cx), cx, |window, cx| {
        let view = cx.new(|cx| Onboarding::new(cx).with_sound());
        let handle = window.window_handle();
        cx.subscribe(&view, move |_, choice: &Chosen, cx| {
            // Recovering an account has no screens yet.
            if *choice != Chosen::NewAccount {
                return;
            }
            if let Some(next) = next.take() {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                    next(cx);
                });
            }
        })
        .detach();
        view
    });
    match opened {
        Ok((_, view)) => Some(view),
        Err(error) => {
            reports::log::line(&format!("onboarding window did not open: {error}"));
            None
        }
    }
}

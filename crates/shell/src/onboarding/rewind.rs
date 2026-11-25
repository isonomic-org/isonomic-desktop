// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Going back on the third and fourth screens: "Decline" on the terms and
//! "Previous step" on "Create your account with Face ID" play what
//! brought the screen on backward, instead of showing the screen before
//! at once.
//!
//! Everything on those screens is placed by the third screen's clock, so
//! going back is that clock running backward: `Onboarding::terms_now`
//! reads it from here for as long as it does. The 3D pieces are drawn
//! again for each time it passes, and what the clock has gone back past
//! is put back as it was before it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ui::onboarding::passkey::at as fourth;
use ui::onboarding::terms::{at, signs};
use ui::{AppContext as _, Context, RenderImage};

use super::laptop::{self, Scene};
use super::passkey::{self, Seal};
use super::props::Props;
use super::stage::Drawn;
use super::{signing, terms, Onboarding, FRAME};

/// Where the clock stops, and what is there.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Land {
    /// The two ways to start.
    Start,
    /// The terms, as they were before they were accepted.
    Terms,
}

/// The clock running backward since `since`: from `from` at its own pace
/// down to `knee`, then faster down to `to`.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Rewind {
    since: Instant,
    from: f32,
    knee: f32,
    to: f32,
    land: Land,
}

impl Rewind {
    /// What the clock reads now.
    pub(super) fn now(&self) -> f32 {
        self.after(self.since.elapsed().as_secs_f32())
    }

    /// What it reads `elapsed` seconds after it turned back.
    fn after(&self, elapsed: f32) -> f32 {
        let knee = self.knee.min(self.from);
        let slow = self.from - knee;
        if elapsed <= slow {
            self.from - elapsed
        } else {
            (knee - (elapsed - slow) * signs::UNDO_RATE).max(self.to)
        }
    }
}

/// Is `t` in the move that starts at `starts` and takes `length`, or just
/// either side of it, where the piece is drawn at rest?
fn within(t: f32, starts: f32, length: f32) -> bool {
    (starts - 0.1..=starts + length + 0.1).contains(&t)
}

#[derive(Default)]
struct Frames {
    laptop: Option<Arc<RenderImage>>,
    clipboard: Option<Arc<RenderImage>>,
    pen: Option<Arc<RenderImage>>,
    seal: Option<Arc<RenderImage>>,
}

/// The pieces that move at `t`, as they are then. `signed` and `passkey`
/// are when the signing and the fourth screen started, if they have.
fn draw(
    (scene, props, seal): (Option<&Scene>, Option<&Props>, Option<&Seal>),
    (signed, passkey): (Option<f32>, Option<f32>),
    t: f32,
    scale: f32,
) -> Frames {
    let since = |started: Option<f32>| started.map(|started| t - started).filter(|s| *s >= 0.);
    let mut frames = Frames::default();
    if let Some(p) = since(passkey) {
        // The fourth screen: the seal.
        frames.seal = seal
            .filter(|_| within(p, fourth::SEAL_FLIES, fourth::FLIGHT))
            .map(|seal| seal.draw(passkey::flight(p).1, scale));
    } else if let Some(s) = since(signed) {
        // The signing: the pages lift and settle, and both leave.
        let settles = signs::CLIPBOARD_LEAVES + signs::LEAVE - signs::RELEASES;
        frames.clipboard = props
            .filter(|_| within(s, 0., signs::LIFT) || within(s, signs::RELEASES, settles))
            .map(|props| {
                let (_, turn) = signing::leaving(s, signs::CLIPBOARD_LEAVES, signs::LEAVE);
                props.clipboard(turn, signing::lift(s), scale)
            });
        frames.pen = props
            .filter(|_| within(s, signs::PEN_LEAVES, signs::PEN_LEAVE))
            .map(|props| {
                let (_, turn) = signing::leaving(s, signs::PEN_LEAVES, signs::PEN_LEAVE);
                props.pen(turn, scale)
            });
    } else {
        // The terms coming on: the laptop, the clipboard and the pen.
        frames.laptop = scene
            .filter(|_| within(t, at::LAPTOP_MOVES, at::LAPTOP_MOVE))
            .map(|scene| {
                let (gone, turn) = terms::laptop_move(t);
                let (_, _, wide, tall) = terms::laptop_box(gone);
                laptop::posed(scene, turn, 1., (wide, tall), scale)
            });
        frames.clipboard = props
            .filter(|_| within(t, at::CLIPBOARD_FLIES, at::FLIGHT))
            .map(|props| props.clipboard(terms::flight(t, at::CLIPBOARD_FLIES).1, 0., scale));
        frames.pen = props
            .filter(|_| within(t, at::PEN_FLIES, at::FLIGHT))
            .map(|props| props.pen(terms::flight(t, at::PEN_FLIES).1, scale));
    }
    frames
}

impl Onboarding {
    /// Is the clock running backward?
    pub(super) fn rewinding(&self) -> Option<&Rewind> {
        self.rewind.as_ref()
    }

    /// Run the clock backward from `from` to `to`, faster below `knee`,
    /// and land.
    pub(super) fn rewind_to(
        &mut self,
        (from, knee, to): (f32, f32, f32),
        land: Land,
        cx: &mut Context<Self>,
    ) {
        let rewind = Rewind {
            since: Instant::now(),
            from,
            knee,
            to,
            land,
        };
        self.rewind = Some(rewind);
        // Whatever was drawing forward knows the clock by when it started,
        // and stops.
        self.terms = Some(rewind.since);
        cx.notify();
        cx.spawn(async move |view, cx| {
            // How long a round of frames takes to draw: they are drawn for
            // that far ahead.
            let mut lead = 1. / 60.;
            loop {
                let Ok(Some((t, scale, models, clocks))) = view.update(cx, |view, cx| {
                    (view.rewind == Some(rewind)).then(|| {
                        let t = rewind.now();
                        view.passed(t, cx);
                        let models = (view.scene.clone(), view.props.clone(), view.seal.clone());
                        (t, view.scale, models, (view.signed, view.passkey))
                    })
                }) else {
                    return;
                };
                if t <= to {
                    break;
                }
                let began = Instant::now();
                let shown = (t - lead).max(to);
                let frames = cx
                    .background_spawn(async move {
                        let (scene, props, seal) = &models;
                        let models = (scene.as_deref(), props.as_deref(), seal.as_deref());
                        draw(models, clocks, shown, scale)
                    })
                    .await;
                lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                let kept = view.update(cx, |view, cx| {
                    if view.rewind != Some(rewind) {
                        return false;
                    }
                    for (slot, new) in [
                        (&mut view.frame, frames.laptop),
                        (&mut view.clipboard, frames.clipboard),
                        (&mut view.pen, frames.pen),
                        (&mut view.seal_frame, frames.seal),
                    ] {
                        if let Some(image) = new {
                            if let Some(old) = slot.replace(Drawn { image, at: shown }) {
                                cx.drop_image(old.image, None);
                            }
                        }
                    }
                    cx.notify();
                    true
                });
                if !matches!(kept, Ok(true)) {
                    return;
                }
                // No faster than the screen shows them.
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
            let _ = view.update(cx, |view, cx| view.land_rewind(cx));
        })
        .detach();
    }

    /// What the clock has gone back past is as it was before it: the
    /// fourth screen is gone, and on the terms the clipboard and the pen
    /// are back under the content.
    fn passed(&mut self, t: f32, cx: &mut Context<Self>) {
        let mut gone = Vec::new();
        if self.passkey.is_some_and(|started| t < started) {
            self.passkey = None;
            gone.push(self.seal_frame.take());
        }
        if self.signed.is_none() {
            if t < at::CLIPBOARD_FLIES - 0.05 {
                gone.push(self.clipboard.take());
            }
            if t < at::PEN_FLIES - 0.05 {
                gone.push(self.pen.take());
            }
        }
        for old in gone.into_iter().flatten() {
            cx.drop_image(old.image, None);
        }
    }

    /// The clock has come to where it was going: it runs forward again
    /// from there.
    fn land_rewind(&mut self, cx: &mut Context<Self>) {
        let Some(rewind) = self.rewind.take() else {
            return;
        };
        match rewind.land {
            Land::Start => self.declined(cx),
            Land::Terms => {
                let now = Instant::now();
                let back = Duration::from_secs_f32(rewind.to);
                self.terms = Some(now.checked_sub(back).unwrap_or(now));
                self.unsign(cx);
            }
        }
    }

    /// Jump to the end of going back, as a test wants it.
    pub fn skip_rewind(&mut self, cx: &mut Context<Self>) {
        self.land_rewind(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_goes_back_at_its_own_pace_then_faster_and_stops() {
        let rewind = Rewind {
            since: Instant::now(),
            from: 10.,
            knee: 8.,
            to: 4.,
            land: Land::Terms,
        };
        assert_eq!(rewind.after(0.), 10.);
        assert_eq!(rewind.after(1.), 9.);
        assert_eq!(rewind.after(2.), 8.);
        assert_eq!(rewind.after(2.5), 8. - 0.5 * signs::UNDO_RATE);
        assert_eq!(rewind.after(60.), 4.);
        // Turned back before the knee: fast from the start.
        let early = Rewind { from: 6., ..rewind };
        assert_eq!(early.after(0.25), 6. - 0.25 * signs::UNDO_RATE);
    }

    #[test]
    fn a_piece_is_drawn_through_its_move_and_at_rest_either_side() {
        assert!(within(1., 1., 2.) && within(3.05, 1., 2.) && within(0.95, 1., 2.));
        assert!(!within(3.2, 1., 2.) && !within(0.8, 1., 2.));
    }
}

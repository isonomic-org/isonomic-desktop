// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding's later steps, from "Continue with Face ID" to the
//! handover: the Face ID sheet, the six steps from "Your account exists."
//! to "Finish", and "Setting up your workspace".
//!
//! Every step comes on the same way: the last step's art turns over and
//! leaves by the left edge, the next panel slides over the last from the
//! right, and the step's art turns over as it comes out from under it.
//! "Previous step" plays the same backward.
//!
//! This is a first pass with nothing behind it: no Face ID is asked for,
//! nothing is created, switched on, connected or saved, and the values
//! shown are the design's samples. Its times are on the third screen's
//! clock.

use std::time::Instant;

use ui::onboarding::kit::at as kit_at;
use ui::onboarding::steps::{self as design, at};
use ui::{motion, AppContext as _, Context};

use super::art::Art;
use super::laptop;
use super::stage::Drawn;
use super::steps::{self, STEPS};
use super::{Onboarding, FRAME};

/// Where the later steps are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flow {
    pub step: usize,
    /// When this step came on.
    pub started: f32,
    /// Did it come on from the step before?
    pub forward: bool,
    /// Is it being left by "Previous step"? Then its coming on plays
    /// backward, and the step before is there when that ends.
    pub back: bool,
}

/// Where the ID card is on its way into the wallet: how far from where
/// it rests over it, in points; how tall it is drawn as it flips over
/// backward, as a share of its height; and whether it is in the wallet
/// yet, behind the pocket, or still over it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slip {
    pub across: f32,
    pub down: f32,
    pub flat: f32,
    pub tucked: bool,
}

impl Slip {
    pub const RESTING: Slip = Slip {
        across: 0.,
        down: 0.,
        flat: 1.,
        tucked: false,
    };
}

/// The card `since` seconds after the press that sends it into the
/// wallet: it flips over backward once as it lifts clear of the wallet's
/// mouth, then slides straight down into it, a little past where it
/// rests, and comes back. It goes up and down, so it turns about its
/// level line, not its upright one.
pub fn slip(since: f32) -> Slip {
    let (left, top, _, _) = design::CARD;
    let up = motion::ease(motion::CARRY, motion::progress(since, 0., at::CARD_LIFTS));
    let tuck = motion::progress(since, at::CARD_LIFTS, at::CARD_TUCKS);
    let down = motion::ease(motion::CARRY, (tuck / 0.7).min(1.));
    let dip = (std::f32::consts::PI * ((tuck - 0.5) / 0.5).clamp(0., 1.)).sin();
    Slip {
        across: (design::CARD_OVER.0 - left) * up
            + (design::CARD_IN.0 - design::CARD_OVER.0) * down,
        down: (design::CARD_OVER.1 - top) * up
            + (design::CARD_IN.1 - design::CARD_OVER.1) * down
            + design::CARD_DIP * dip,
        flat: turned(0.5 * up),
        tucked: since >= at::CARD_LIFTS,
    }
}

/// How far the old art has gone (0 to 1), `f` seconds into a step.
pub fn left(f: f32) -> f32 {
    motion::ease(motion::DEPART, motion::progress(f, 0., at::LEAVE))
}

/// How far in the new art is (0 to 1).
pub fn arrived(f: f32) -> f32 {
    motion::ease(
        motion::CARRY,
        motion::progress(f, at::ART_ARRIVES, at::FLIGHT),
    )
}

/// How far the laptop is toward the middle `g` seconds into "Setting up
/// your workspace" (0 to 1), and the turn it has left to make, in degrees.
pub fn centring(g: f32) -> (f32, f32) {
    let there = motion::ease(
        motion::CARRY,
        motion::progress(g, at::LAPTOP_CENTRES, at::CENTRE),
    );
    (
        there,
        (1. - there) * 360. * ui::onboarding::steps::SETUP_TURNS,
    )
}

/// How wide something that has turned `share` of the way over is drawn,
/// as a share of its width: it narrows to its edge and widens again.
pub fn turned(share: f32) -> f32 {
    (std::f32::consts::TAU * share)
        .cos()
        .abs()
        .max(ui::onboarding::steps::EDGE_ON)
}

impl Onboarding {
    /// The later steps' place and seconds into it, at `t`. A step being
    /// left by "Previous step" is that step coming on, with its seconds
    /// counting down; when they run out, the step before is there, at rest.
    pub fn flow_at(&self, t: f32) -> Option<(Flow, f32)> {
        let flow = self.flow?;
        if !flow.back {
            return Some((flow, t - flow.started - Self::waits(flow)));
        }
        let coming = Flow {
            forward: true,
            back: false,
            ..flow
        };
        let left = self.span(flow) - (t - flow.started);
        if left > 0. {
            Some((coming, left - Self::waits(coming)))
        } else {
            Self::before(flow, t).map(|rests| (rests, at::END))
        }
    }

    /// The step before `flow`'s, at rest since `t`; none before the first.
    fn before(flow: Flow, t: f32) -> Option<Flow> {
        flow.step.checked_sub(1).map(|step| Flow {
            step,
            started: t,
            forward: false,
            back: false,
        })
    }

    /// How long a step takes to come on, and so to be left backward: its
    /// wait, its motion, what its art does after, and, for the confirmed
    /// kit, its unlocking.
    fn span(&self, flow: Flow) -> f32 {
        let coming = Flow {
            forward: true,
            back: false,
            ..flow
        };
        let unlocks = if self.unlocking && flow.step == steps::CONFIRM {
            kit_at::CONFIRMED
        } else {
            0.
        };
        Self::waits(coming) + at::END + Self::lingers(coming) + unlocks
    }

    /// Once a step has been left backward, the step before is where the
    /// later steps are. Called at every draw.
    pub(super) fn land(&mut self) {
        let (Some(flow), Some(t)) = (self.flow, self.terms_now()) else {
            return;
        };
        if flow.back && t - flow.started >= self.span(flow) {
            self.flow = Self::before(flow, t);
            self.unlocking = false;
            self.wants_words = flow.step == steps::CONFIRM + 1 && !self.locked;
        }
    }

    /// How long a step waits before it comes on: the step after "Your
    /// account exists." waits for the card to slip into the wallet.
    fn waits(flow: Flow) -> f32 {
        if flow.forward && flow.step == steps::KIT {
            at::SLIP
        } else {
            0.
        }
    }

    /// Where the ID card is at `t`.
    pub(super) fn slipped(&self, t: f32) -> Slip {
        match self.flow_at(t) {
            Some((flow, f)) if Self::waits(flow) > 0. => slip(f + Self::waits(flow)),
            _ => Slip::RESTING,
        }
    }

    /// How long a step's art takes to come to rest after the rest of the
    /// step has: on "Your account exists." the card follows the wallet,
    /// and on "Save your recovery kit" the holder opens.
    fn lingers(flow: Flow) -> f32 {
        match flow.step {
            steps::CREATED => at::CARD_FOLLOWS,
            // The holder opens after it has come.
            steps::KIT => kit_at::OPENS + kit_at::OPEN - at::END,
            _ => 0.,
        }
    }

    fn settled_flow(&self) -> bool {
        match (self.flow, self.terms_now()) {
            (Some(flow), Some(t)) => {
                !flow.back
                    && (!flow.forward
                        || t - flow.started >= at::END + Self::waits(flow) + Self::lingers(flow))
            }
            _ => true,
        }
    }

    /// "Continue with Face ID": the sheet comes up as the system's would,
    /// and the first of the later steps follows.
    pub fn continue_with_face_id(&mut self, cx: &mut Context<Self>) {
        let (Some(_), Some(t), None, None) =
            (self.passkey, self.terms_now(), self.asking, self.flow)
        else {
            return;
        };
        if self.rewinding().is_some() {
            return;
        }
        self.asking = Some(t);
        cx.notify();
        cx.spawn(async move |view, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs_f32(at::ASKING))
                .await;
            let _ = view.update(cx, |view, cx| {
                if view.asking == Some(t) {
                    view.answer_face_id(cx);
                }
            });
        })
        .detach();
    }

    /// The sheet's "Cancel".
    pub fn cancel_face_id(&mut self, cx: &mut Context<Self>) {
        self.asking = None;
        cx.notify();
    }

    fn answer_face_id(&mut self, cx: &mut Context<Self>) {
        let Some(t) = self.terms_now() else {
            return;
        };
        self.asking = None;
        self.answered = Some(t);
        self.flow = Some(Flow {
            step: steps::CREATED,
            started: t,
            forward: true,
            back: false,
        });
        cx.notify();
    }

    /// Is the step's forward button waiting on the person? On "Save your
    /// recovery kit" it is, until the kit is ticked as stored.
    pub(super) fn held_back(&self) -> bool {
        self.flow.is_some_and(|flow| flow.step == steps::KIT) && !self.stored
    }

    /// Tick the kit as stored somewhere safe, or not.
    pub fn store_kit(&mut self, stored: bool, cx: &mut Context<Self>) {
        self.stored = stored;
        cx.notify();
    }

    /// A step's forward button: the next step, or after the last,
    /// "Setting up your workspace" and the handover.
    pub fn next_step(&mut self, cx: &mut Context<Self>) {
        let (Some(flow), Some(t), None) = (self.flow, self.terms_now(), self.finishing) else {
            return;
        };
        if !self.settled_flow() {
            return;
        }
        if self.held_back() {
            return;
        }
        if flow.step == steps::CONFIRM && !self.locked {
            // Not before its three words are typed back.
            self.confirm_words(t, cx);
            return;
        }
        if flow.step + 1 < STEPS.len() {
            self.advance(t, cx);
            return;
        }
        self.finishing = Some(t);
        cx.notify();
        cx.spawn(async move |view, cx| {
            // The laptop turns over on its way to the middle: its frames
            // are drawn for the time they will reach the screen.
            let mut last = f32::MIN;
            let mut lead = 1. / 60.;
            loop {
                let Ok(Some((g, now, scale, scene))) = view.update(cx, |view, _| {
                    let now = view.terms_now()?;
                    Some((now - t, now, view.scale, view.scene.clone()))
                }) else {
                    return;
                };
                if g >= at::SETTING_UP {
                    break;
                }
                let began = Instant::now();
                let shown = super::stage::snap(g + lead);
                let drawn_for = now + shown - g;
                let ends = at::LAPTOP_CENTRES + at::CENTRE;
                if let (true, Some(scene)) = (shown >= at::LAPTOP_CENTRES && last < ends, scene) {
                    let (_, turn) = centring(shown);
                    let (_, _, wide, tall) = ui::onboarding::terms::LAPTOP;
                    let image = cx
                        .background_spawn(async move {
                            laptop::posed(
                                &scene,
                                (turn, laptop::Flip::Right),
                                1.,
                                (wide, tall),
                                scale,
                            )
                        })
                        .await;
                    last = shown;
                    lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                    let kept = view.update(cx, |view, cx| {
                        let frame = Drawn {
                            image,
                            at: drawn_for,
                        };
                        super::stage::put(&mut view.frame, frame, cx);
                        cx.notify();
                    });
                    if kept.is_err() {
                        return;
                    }
                }
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
            let _ = view.update(cx, |view, cx| view.hand_over(cx));
        })
        .detach();
    }

    /// Bring the next step on at `t`.
    pub(super) fn advance(&mut self, t: f32, cx: &mut Context<Self>) {
        let Some(flow) = self.flow.filter(|flow| flow.step + 1 < STEPS.len()) else {
            return;
        };
        self.flow = Some(Flow {
            step: flow.step + 1,
            started: t,
            forward: true,
            back: false,
        });
        // Past "Confirm your recovery kit", the kit stays locked.
        self.locked |= flow.step == steps::CONFIRM;
        self.confirmed = None;
        self.wants_words = flow.step + 1 == steps::CONFIRM;
        cx.notify();
    }

    /// A step's "Previous step": its coming on plays backward, and the
    /// step before is there when that ends. The locked kit is unlocked on
    /// the way back out of "Confirm your recovery kit".
    pub fn back_step(&mut self, cx: &mut Context<Self>) {
        let (Some(flow), Some(t), None) = (self.flow, self.terms_now(), self.finishing) else {
            return;
        };
        if !self.settled_flow() {
            return;
        }
        if flow.step == steps::CONFIRM {
            self.unlocking = std::mem::take(&mut self.locked);
            self.unconfirm();
        }
        self.flow = Some(Flow {
            started: t,
            forward: false,
            back: true,
            ..flow
        });
        cx.notify();
    }

    /// Is "Setting up your workspace" up? It draws the laptop itself.
    pub(super) fn is_setting_up(&self) -> bool {
        self.finishing.is_some()
    }

    /// Jump past whatever the later steps are waiting on or moving
    /// through, as a test wants it.
    pub fn skip_flow(&mut self, cx: &mut Context<Self>) {
        if self.asking.is_some() {
            self.answer_face_id(cx);
        }
        if self.finishing.is_some() {
            self.hand_over(cx);
        }
        // The confirmed kit is locked at once.
        if let (Some(_), Some(t), Some(steps::CONFIRM)) = (
            self.confirmed,
            self.terms_now(),
            self.flow.map(|flow| flow.step),
        ) {
            self.advance(t, cx);
        }
        // The sheet is gone at once, not faded.
        self.answered = None;
        self.flow = match (self.flow, self.terms_now()) {
            (Some(flow), Some(t)) if flow.back => Self::before(flow, t),
            (Some(flow), _) => Some(Flow {
                forward: false,
                ..flow
            }),
            _ => None,
        };
        self.unlocking = false;
        cx.notify();
    }

    /// Flip switch `index` of the list on step `step`.
    pub(super) fn flip(&mut self, step: usize, index: usize, cx: &mut Context<Self>) {
        let switches: &mut [bool] = match step {
            steps::MODELS => &mut self.models,
            steps::LINKS => &mut self.links,
            _ => &mut self.residency,
        };
        if let Some(on) = switches.get_mut(index) {
            *on = !*on;
        }
        cx.notify();
    }

    /// Read the later steps' art, off the foreground thread.
    pub(super) fn load_art(cx: &mut Context<Self>) {
        cx.spawn(async move |view, cx| {
            let art = cx.background_spawn(async { Art::load() }).await;
            let _ = view.update(cx, |view, _| view.art = Some(std::sync::Arc::new(art)));
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_art_is_gone_before_the_new_arrives() {
        assert!((left(at::LEAVE) - 1.).abs() < 1e-3);
        assert!(arrived(at::LEAVE) < 0.05);
        assert!((arrived(at::END) - 1.).abs() < 1e-3);
    }

    #[test]
    fn the_card_lifts_over_the_wallet_and_ends_tucked_in_it() {
        let rest = slip(0.);
        assert!(rest.down.abs() < 0.5 && !rest.tucked && (rest.flat - 1.).abs() < 1e-3);
        let clear = slip(at::CARD_LIFTS);
        assert!((design::CARD.1 + clear.down - design::CARD_OVER.1).abs() < 0.5);
        assert!(clear.tucked && (clear.flat - 1.).abs() < 1e-3);
        // On the way it is seen edge-on.
        let thinnest = (0..=100)
            .map(|step| slip(at::CARD_LIFTS * step as f32 / 100.).flat)
            .fold(1., f32::min);
        assert!(thinnest < 0.2);
        let rests = slip(at::SLIP);
        assert!((design::CARD.1 + rests.down - design::CARD_IN.1).abs() < 0.5);
        assert!((rests.flat - 1.).abs() < 1e-3);
    }

    #[test]
    fn a_turn_narrows_to_an_edge_and_comes_back() {
        assert!((turned(0.) - 1.).abs() < 1e-3);
        assert!(turned(0.25) < 0.05);
        assert!((turned(0.5) - 1.).abs() < 1e-3);
        assert!((turned(1.) - 1.).abs() < 1e-3);
    }
}

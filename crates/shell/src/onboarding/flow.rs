// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding's later steps, from "Continue with Face ID" to the
//! handover: the Face ID sheet, the six steps from "Your account exists."
//! to "Finish", and "Setting up your workspace".
//!
//! Every step comes on the same way: the last step's art turns over and
//! leaves by the left edge, the next panel slides over the last from the
//! right, and the step's art turns over as it comes out from under it.
//!
//! This is a first pass with nothing behind it: no Face ID is asked for,
//! nothing is created, switched on, connected or saved, and the values
//! shown are the design's samples. Its times are on the third screen's
//! clock.

use std::time::Instant;

use ui::onboarding::steps::at;
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
    /// Did it come on from the step before? Coming back has no motion.
    pub forward: bool,
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
    /// The later steps' place and seconds into it, at `t`.
    pub fn flow_at(&self, t: f32) -> Option<(Flow, f32)> {
        self.flow.map(|flow| (flow, t - flow.started))
    }

    fn settled_flow(&self) -> bool {
        match (self.flow, self.terms_now()) {
            (Some(flow), Some(t)) => !flow.forward || t - flow.started >= at::END,
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
        });
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
        if flow.step + 1 < STEPS.len() {
            self.flow = Some(Flow {
                step: flow.step + 1,
                started: t,
                forward: true,
            });
            cx.notify();
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
                let (shown, drawn_for) = (g + lead, now + lead);
                let ends = at::LAPTOP_CENTRES + at::CENTRE;
                if let (true, Some(scene)) = (shown >= at::LAPTOP_CENTRES && last < ends, scene) {
                    let (_, turn) = centring(shown);
                    let (_, _, wide, tall) = ui::onboarding::terms::LAPTOP;
                    let image = cx
                        .background_spawn(async move {
                            laptop::posed(&scene, turn, 1., (wide, tall), scale)
                        })
                        .await;
                    last = shown;
                    lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                    let kept = view.update(cx, |view, cx| {
                        let frame = Drawn {
                            image,
                            at: drawn_for,
                        };
                        if let Some(old) = view.frame.replace(frame) {
                            cx.drop_image(old.image, None);
                        }
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

    /// A step's "Previous step": the step before, as it rests.
    pub fn back_step(&mut self, cx: &mut Context<Self>) {
        let (Some(flow), Some(t), None) = (self.flow, self.terms_now(), self.finishing) else {
            return;
        };
        if !self.settled_flow() {
            return;
        }
        self.flow = flow.step.checked_sub(1).map(|step| Flow {
            step,
            started: t,
            forward: false,
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
        // The sheet is gone at once, not faded.
        self.answered = None;
        if let Some(flow) = self.flow.as_mut() {
            flow.forward = false;
        }
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
    fn a_turn_narrows_to_an_edge_and_comes_back() {
        assert!((turned(0.) - 1.).abs() < 1e-3);
        assert!(turned(0.25) < 0.05);
        assert!((turned(0.5) - 1.).abs() < 1e-3);
        assert!((turned(1.) - 1.).abs() < 1e-3);
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the recovery kit's holder does through "Save your recovery kit"
//! and "Confirm your recovery kit", and the words typed back on the
//! second.
//!
//! It comes out from under the first step's panel closed, opens, and its
//! letters turn into place. On the second step it turns its back. A wrong
//! word shakes it. The right words close it, its mark up and its loops
//! toward the person; the padlock comes, its body snaps onto its shackle,
//! and they leave by the left edge as the next step comes on.
//!
//! Its times are on the third screen's clock, like every later step's.

use std::time::Instant;

use ui::onboarding::kit::{self as design, at};
use ui::onboarding::steps::at as step;
use ui::onboarding::terms as shared;
use ui::{motion, AppContext as _, Context, KeyDownEvent};

use super::flow::{self, Flow};
use super::kit::{self, Pose};
use super::stage::Drawn;
use super::steps;
use super::Onboarding;

/// The longest a typed word may be.
const LONGEST: usize = 12;

/// How far the holder is shaken to the right, `s` seconds after a wrong
/// word, in points.
pub fn shake(s: f32) -> f32 {
    let share = motion::progress(s, 0., at::SHAKE);
    if share <= 0. || share >= 1. {
        return 0.;
    }
    let (far, times) = design::SHAKE;
    far * (std::f32::consts::TAU * times * share).sin() * (1. - share)
}

impl Onboarding {
    /// How the holder lies at `t`, while it is in the sidebar.
    pub(super) fn kit_pose(&self, t: f32) -> Option<Pose> {
        let (flow, f) = self.flow_at(t)?;
        let turned = kit::spinning(self.kit.as_ref().map_or(0, |kit| kit.tiles()));
        let ease = |starts: f32, length: f32| {
            motion::ease(motion::CARRY, motion::progress(f, starts, length))
        };
        match (flow.step, flow.forward, self.confirmed) {
            (steps::KIT, true, _) => {
                let opened = ease(at::OPENS, at::OPEN);
                Some(Pose {
                    turn: (1. - flow::arrived(f)) * 360.,
                    away: 0.,
                    fold: 180. * (1. - opened),
                    risen: 1. - opened,
                    spun: (f - at::SPINS).clamp(0., turned),
                })
            }
            (steps::KIT, false, _) => Some(Pose::OPEN),
            (steps::CONFIRM, _, Some(confirmed)) => {
                let closed = motion::ease(
                    motion::CARRY,
                    motion::progress(t - confirmed, 0., at::CLOSE),
                );
                Some(Pose {
                    away: 180. * (1. - closed),
                    fold: 180. * closed,
                    risen: closed,
                    ..Pose::OPEN
                })
            }
            (steps::CONFIRM, true, None) => Some(Pose {
                away: 180. * ease(at::TURNS_AWAY, at::TURN),
                ..Pose::OPEN
            }),
            (steps::CONFIRM, false, None) => Some(Pose::AWAY),
            (steps::MODELS, true, _) if f < step::LEAVE => Some(Pose::CLOSED),
            _ => None,
        }
    }

    /// How far up its shackle the padlock's body is at `t` (0 to 1),
    /// while the padlock is in the sidebar.
    pub(super) fn lock_clip(&self, t: f32) -> Option<f32> {
        let (flow, f) = self.flow_at(t)?;
        match (flow.step, flow.forward) {
            (steps::CONFIRM, _) => {
                let s = t - self.confirmed?;
                (s >= at::LOCK_COMES).then(|| {
                    motion::ease(
                        motion::ARRIVE,
                        motion::progress(s, at::LOCK_CLIPS, at::CLIP),
                    )
                })
            }
            (steps::MODELS, true) if f < step::LEAVE => Some(1.),
            _ => None,
        }
    }

    /// Where the holder's picture is carried at `t`: how far to the right
    /// of its place, in points, and how wide it is drawn, as a share of
    /// its width. It comes out from under the panel and leaves by the left
    /// edge as every step's art does.
    pub(super) fn kit_carried(&self, t: f32) -> (f32, f32) {
        match self.flow_at(t) {
            Some((
                Flow {
                    step: steps::KIT,
                    forward: true,
                    ..
                },
                f,
            )) => {
                let from =
                    shared::SIDEBAR + shared::TUCK - (design::CENTRE.0 - design::CANVAS.0 / 2.);
                (from * (1. - flow::arrived(f)), 1.)
            }
            Some((
                Flow {
                    step: steps::MODELS,
                    forward: true,
                    ..
                },
                f,
            )) => {
                let gone = flow::left(f);
                (-(shared::SIDEBAR + 40.) * gone, flow::turned(gone))
            }
            _ => (0., 1.),
        }
    }

    /// How far the padlock still is from its place at `t`, in points to
    /// the right: it comes out from under the panel.
    pub(super) fn lock_coming(&self, t: f32) -> f32 {
        let Some(confirmed) = self.confirmed else {
            return 0.;
        };
        let came = motion::ease(
            motion::CARRY,
            motion::progress(t - confirmed, at::LOCK_COMES, at::LOCK_FLIGHT),
        );
        (shared::SIDEBAR + shared::TUCK - design::CENTRE.0 + design::LOCK_CANVAS.0) * (1. - came)
    }

    /// Draw the holder's and the padlock's next frames if how they lie
    /// has changed since their last. Called at every draw of the window,
    /// so the frames keep the screen's own pace and wait on no timer; one
    /// drawing is in hand at a time.
    pub(super) fn draw_kit(&mut self, cx: &mut Context<Self>) {
        if self.kit_drawing {
            return;
        }
        let at = self.terms_now().map(|t| t + self.kit_lead);
        let holder = at.and_then(|at| self.kit_pose(at));
        let lock = at.and_then(|at| self.lock_clip(at));
        // Whatever has gone from the sidebar is let go.
        if holder.is_none() {
            if let Some((old, _)) = self.kit_frame.take() {
                cx.drop_image(old.image, None);
            }
        }
        if lock.is_none() {
            if let Some((old, _)) = self.lock_frame.take() {
                cx.drop_image(old.image, None);
            }
        }
        let shown = self.kit_frame.as_ref().map(|(_, pose)| *pose);
        let clipped = self.lock_frame.as_ref().map(|(_, clip)| *clip);
        let holder = holder.filter(|pose| shown != Some(*pose));
        let lock = lock.filter(|clip| clipped != Some(*clip));
        let (Some(at), Some(kit), true) =
            (at, self.kit.clone(), holder.is_some() || lock.is_some())
        else {
            return;
        };
        self.kit_drawing = true;
        let scale = self.scale;
        cx.spawn(async move |view, cx| {
            let began = Instant::now();
            let (holder, lock) = cx
                .background_spawn(async move {
                    (
                        holder.map(|pose| (kit.holder(pose, scale), pose)),
                        lock.map(|clip| (kit.lock(clip, scale), clip)),
                    )
                })
                .await;
            let _ = view.update(cx, |view, cx| {
                view.kit_drawing = false;
                view.kit_lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                if let Some((image, pose)) = holder {
                    if let Some((old, _)) = view.kit_frame.replace((Drawn { image, at }, pose)) {
                        cx.drop_image(old.image, None);
                    }
                }
                if let Some((image, clip)) = lock {
                    if let Some((old, _)) = view.lock_frame.replace((Drawn { image, at }, clip)) {
                        cx.drop_image(old.image, None);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Is typed word `index` the word the kit has in that place? Capital
    /// letters and the spaces around it don't matter.
    fn matches(&self, index: usize) -> bool {
        self.typed[index].trim().to_lowercase() == steps::WORDS[steps::ASKED[index] - 1]
    }

    /// How many of the three words are typed.
    pub(super) fn words_typed(&self) -> usize {
        self.typed
            .iter()
            .filter(|word| !word.trim().is_empty())
            .count()
    }

    /// "Continue" on "Confirm your recovery kit": a wrong word shakes the
    /// holder; the right three close and lock it, and then the next step
    /// comes on.
    pub(super) fn confirm_words(&mut self, t: f32, cx: &mut Context<Self>) {
        if self.confirmed.is_some() {
            return;
        }
        self.wrong = [0, 1, 2].map(|index| !self.matches(index));
        if self.wrong.contains(&true) {
            self.shaken = Some(t);
            if let Some(first) = self.wrong.iter().position(|wrong| *wrong) {
                self.typing = first;
            }
            cx.notify();
            return;
        }
        self.confirmed = Some(t);
        cx.notify();
        cx.spawn(async move |view, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs_f32(at::CONFIRMED))
                .await;
            let _ = view.update(cx, |view, cx| {
                if let (Some(now), true) = (view.terms_now(), view.confirmed == Some(t)) {
                    view.advance(now, cx);
                }
            });
        })
        .detach();
    }

    /// Start typing word `index`.
    pub(super) fn type_word(&mut self, index: usize, cx: &mut Context<Self>) {
        self.typing = index;
        cx.notify();
    }

    /// A key, while the words have the keyboard.
    pub(super) fn word_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.confirmed.is_some() {
            return;
        }
        let keys = &event.keystroke;
        if keys.modifiers.platform || keys.modifiers.control || keys.modifiers.alt {
            return;
        }
        match keys.key.as_str() {
            "backspace" => {
                self.typed[self.typing].pop();
                self.wrong[self.typing] = false;
            }
            "tab" if keys.modifiers.shift => self.typing = (self.typing + 2) % 3,
            "tab" => self.typing = (self.typing + 1) % 3,
            "enter" => {
                self.next_step(cx);
                return;
            }
            _ => {
                let letters = keys.key_char.as_deref().unwrap_or_default();
                let word = &mut self.typed[self.typing];
                for letter in letters.chars().filter(|letter| letter.is_alphabetic()) {
                    if word.chars().count() < LONGEST {
                        word.push(letter);
                    }
                }
                self.wrong[self.typing] = false;
            }
        }
        cx.stop_propagation();
        cx.notify();
    }

    /// Type the three words as the kit has them, as a test wants it.
    pub fn fill_words(&mut self, cx: &mut Context<Self>) {
        self.typed = steps::ASKED.map(|place| steps::WORDS[place - 1].to_owned());
        self.wrong = [false; 3];
        cx.notify();
    }

    /// The typed words and what the last press made of them are forgotten
    /// when the step is left backward.
    pub(super) fn unconfirm(&mut self) {
        self.confirmed = None;
        self.shaken = None;
        self.wrong = [false; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shake_starts_and_ends_at_rest() {
        assert_eq!(shake(0.), 0.);
        assert_eq!(shake(at::SHAKE), 0.);
        let far = (1..40)
            .map(|step| shake(at::SHAKE * step as f32 / 40.).abs())
            .fold(0., f32::max);
        assert!(far > 3. && far <= design::SHAKE.0);
    }
}

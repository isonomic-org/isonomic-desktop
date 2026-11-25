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

use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

use ui::onboarding::kit::{self as design, at};
use ui::onboarding::steps::at as step;
use ui::onboarding::terms as shared;
use ui::{motion, Context, KeyDownEvent, RenderImage};

use super::flow::{self, Flow};
use super::kit::{self, Kit, Pose};
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

/// A drawing asked of the easel: the time on the third screen's clock it
/// is for, when it was asked, the screen's pixels to a point, and how the
/// holder and the padlock are to lie, each only if it has changed.
struct Ask {
    at: f32,
    asked: Instant,
    scale: f32,
    holder: Option<Pose>,
    lock: Option<f32>,
}

/// What came of it.
pub(super) struct Done {
    at: f32,
    asked: Instant,
    holder: Option<(Arc<RenderImage>, Pose)>,
    lock: Option<(Arc<RenderImage>, f32)>,
}

/// The thread that draws the holder's and the padlock's frames, one
/// drawing at a time. The window asks at one draw and takes what is
/// finished at a later one, so a frame waits on nothing but the drawing
/// itself: not on a timer, nor on the foreground thread finding a moment
/// between two draws of a busy window.
pub struct Easel {
    asks: mpsc::Sender<Ask>,
    done: Arc<Mutex<Option<Done>>>,
    /// Is a drawing in hand?
    busy: bool,
    /// How long the last drawing took to come back: the next is drawn
    /// for that far ahead.
    lead: f32,
}

impl Easel {
    fn start(kit: Arc<Kit>) -> Easel {
        let (asks, asked) = mpsc::channel::<Ask>();
        let done = Arc::new(Mutex::new(None));
        let finished = Arc::clone(&done);
        // It ends when the window, which holds the other end, has gone.
        std::thread::spawn(move || {
            while let Ok(ask) = asked.recv() {
                let drawn = Done {
                    at: ask.at,
                    asked: ask.asked,
                    holder: ask.holder.map(|pose| (kit.holder(pose, ask.scale), pose)),
                    lock: ask.lock.map(|clip| (kit.lock(clip, ask.scale), clip)),
                };
                if let Ok(mut done) = finished.lock() {
                    *done = Some(drawn);
                }
            }
        });
        Easel {
            asks,
            done,
            busy: false,
            lead: 1. / 60.,
        }
    }

    /// The drawing in hand, if it is finished.
    fn finished(&mut self) -> Option<Done> {
        let done = self.done.lock().ok()?.take()?;
        self.busy = false;
        self.lead = done.asked.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
        Some(done)
    }

    /// Ask for a drawing, unless one is in hand.
    fn ask(&mut self, at: f32, scale: f32, holder: Option<Pose>, lock: Option<f32>) {
        if self.busy {
            return;
        }
        let ask = Ask {
            at,
            asked: Instant::now(),
            scale,
            holder,
            lock,
        };
        self.busy = self.asks.send(ask).is_ok();
    }
}

impl Onboarding {
    /// Seconds into the confirmed kit's closing and locking, at `t`: from
    /// the press that confirmed it, or counting down as "Previous step"
    /// leaves the step and unlocks it.
    fn confirming(&self, t: f32) -> Option<f32> {
        if let Some(confirmed) = self.confirmed {
            return Some(t - confirmed);
        }
        match self.flow_at(t)? {
            (
                Flow {
                    step: steps::CONFIRM,
                    forward: true,
                    ..
                },
                f,
            ) if self.unlocking && f > step::END => Some(f - step::END),
            _ => None,
        }
    }

    /// How the holder lies at `t`, while it is in the sidebar.
    pub(super) fn kit_pose(&self, t: f32) -> Option<Pose> {
        let (flow, f) = self.flow_at(t)?;
        if let Some(s) = self.confirming(t) {
            let closed = motion::ease(motion::CARRY, motion::progress(s, 0., at::CLOSE));
            return Some(Pose {
                away: 180. * (1. - closed),
                fold: 180. * closed,
                risen: closed,
                ..Pose::OPEN
            });
        }
        let turned = kit::spinning(self.kit.as_ref().map_or(0, |kit| kit.tiles()));
        let ease = |starts: f32, length: f32| {
            motion::ease(motion::CARRY, motion::progress(f, starts, length))
        };
        match (flow.step, flow.forward) {
            (steps::KIT, true) => {
                let opened = ease(at::OPENS, at::OPEN);
                Some(Pose {
                    turn: (1. - flow::arrived(f)) * 360.,
                    away: 0.,
                    fold: 180. * (1. - opened),
                    risen: 1. - opened,
                    spun: (f - at::SPINS).clamp(0., turned),
                })
            }
            (steps::KIT, false) => Some(Pose::OPEN),
            (steps::CONFIRM, true) => Some(Pose {
                away: 180. * ease(at::TURNS_AWAY, at::TURN),
                ..Pose::OPEN
            }),
            // Come back to from a later step, it is still locked.
            (steps::CONFIRM, false) if self.locked => Some(Pose::CLOSED),
            (steps::CONFIRM, false) => Some(Pose::AWAY),
            (steps::MODELS, true) if f < step::LEAVE => Some(Pose::CLOSED),
            _ => None,
        }
    }

    /// How far up its shackle the padlock's body is at `t` (0 to 1),
    /// while the padlock is in the sidebar.
    pub(super) fn lock_clip(&self, t: f32) -> Option<f32> {
        let (flow, f) = self.flow_at(t)?;
        if let Some(s) = self.confirming(t) {
            return (s >= at::LOCK_COMES).then(|| {
                motion::ease(
                    motion::ARRIVE,
                    motion::progress(s, at::LOCK_CLIPS, at::CLIP),
                )
            });
        }
        match (flow.step, flow.forward) {
            (steps::CONFIRM, false) if self.locked => Some(1.),
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
        let Some(s) = self.confirming(t) else {
            return 0.;
        };
        let came = motion::ease(
            motion::CARRY,
            motion::progress(s, at::LOCK_COMES, at::LOCK_FLIGHT),
        );
        (shared::SIDEBAR + shared::TUCK - design::CENTRE.0 + design::LOCK_CANVAS.0) * (1. - came)
    }

    /// Take the frame the easel has finished, and ask it for the next if
    /// how the holder or the padlock lies has changed since. Called at
    /// every draw of the window, so the frames keep the screen's own pace.
    pub(super) fn draw_kit(&mut self, cx: &mut Context<Self>) {
        if self.easel.is_none() {
            self.easel = self.kit.clone().map(Easel::start);
        }
        if let Some(done) = self.easel.as_mut().and_then(Easel::finished) {
            let at = done.at;
            if let Some((image, pose)) = done.holder {
                if let Some((old, _)) = self.kit_frame.replace((Drawn { image, at }, pose)) {
                    cx.drop_image(old.image, None);
                }
            }
            if let Some((image, clip)) = done.lock {
                if let Some((old, _)) = self.lock_frame.replace((Drawn { image, at }, clip)) {
                    cx.drop_image(old.image, None);
                }
            }
        }
        let lead = self.easel.as_ref().map_or(0., |easel| easel.lead);
        let at = self.terms_now().map(|t| t + lead);
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
        if let (Some(at), Some(easel), true) =
            (at, self.easel.as_mut(), holder.is_some() || lock.is_some())
        {
            easel.ask(at, self.scale, holder, lock);
        }
    }

    /// Is typed word `index` the word the kit has in that place? Capital
    /// letters and the spaces around it don't matter.
    fn matches(&self, index: usize) -> bool {
        self.typed[index].trim().to_lowercase() == steps::WORDS[steps::ASKED[index] - 1]
    }

    /// Have the three words been confirmed?
    pub(super) fn is_confirmed(&self) -> bool {
        self.confirmed.is_some() || self.locked
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
        if self.is_confirmed() {
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
        if self.is_confirmed() {
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
    fn the_easel_draws_what_it_is_asked_one_drawing_at_a_time() {
        let mut easel = Easel::start(Arc::new(Kit::load().unwrap()));
        assert!(easel.finished().is_none());
        easel.ask(1., 1., Some(Pose::CLOSED), Some(1.));
        assert!(easel.busy);
        // Not heard while the first is in hand.
        easel.ask(2., 1., Some(Pose::OPEN), None);
        let done = (0..500)
            .find_map(|_| {
                std::thread::sleep(std::time::Duration::from_millis(10));
                easel.finished()
            })
            .expect("the easel never finished");
        assert_eq!(done.at, 1.);
        assert_eq!(done.holder.map(|(_, pose)| pose), Some(Pose::CLOSED));
        assert_eq!(done.lock.map(|(_, clip)| clip), Some(1.));
        assert!(!easel.busy && easel.finished().is_none());
    }

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

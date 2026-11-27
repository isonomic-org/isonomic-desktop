// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What "Accept and continue" sets going on the third screen: the
//! clipboard's pages lift by their corner and curl back off the last page,
//! the pen goes to that corner and signs "hello", the pages are let go, the
//! clipboard turns over and leaves by the left edge, the pen follows, and
//! the fourth screen comes on (`passkey`).
//!
//! Its times are seconds since the press, which is itself a time on the
//! third screen's clock, so every drawn frame is still placed by one clock.

use std::sync::Arc;
use std::time::Instant;

use ui::onboarding::terms::{self as design, at, signs};
use ui::{motion, AppContext as _, Context, RenderImage};

use super::props::Props;
use super::stage::{self, Drawn};
use super::{signature, Chosen, Onboarding, FRAME};

/// How far the pages are lifted `s` seconds in, from 0 (lying flat) to 1.
pub fn lift(s: f32) -> f32 {
    let up = motion::ease(motion::CARRY, motion::progress(s, 0., signs::LIFT));
    let down = motion::ease(
        motion::CARRY,
        motion::progress(s, signs::RELEASES, signs::RELEASE),
    );
    up - down
}

/// The share of the signature written.
pub fn written(s: f32) -> f32 {
    motion::progress(s, signs::WRITES, signs::WRITE)
}

/// How far the pen is from its rest toward the page (0 to 1): it goes to
/// the signature's start, stays on the line while it writes, and returns.
pub fn reached(s: f32) -> f32 {
    let to = motion::ease(
        motion::CARRY,
        motion::progress(s, signs::PEN_REACHES, signs::REACH),
    );
    let back = motion::ease(
        motion::CARRY,
        motion::progress(s, signs::PEN_RETURNS, signs::REACH),
    );
    to - back
}

/// How far something that sets off at `starts` and takes `length` has
/// gone toward the left edge (0 to 1), and how far it has turned over.
pub fn leaving(s: f32, starts: f32, length: f32) -> (f32, f32) {
    let gone = motion::ease(motion::DEPART, motion::progress(s, starts, length));
    (gone, gone * 360. * design::TURNS)
}

/// The frames due between the last drawn time and `s`.
fn draw(props: &Props, last: f32, s: f32, scale: f32) -> [Option<Arc<RenderImage>>; 2] {
    let due = |starts: f32, ends: f32| s >= starts && last < ends;
    let lifting = due(0., signs::LIFT);
    let settling = due(signs::RELEASES, signs::CLIPBOARD_LEAVES);
    let going = due(
        signs::CLIPBOARD_LEAVES,
        signs::CLIPBOARD_LEAVES + signs::LEAVE,
    );
    let clipboard = (lifting || settling || going).then(|| {
        let (_, turn) = leaving(s, signs::CLIPBOARD_LEAVES, signs::LEAVE);
        props.clipboard(turn, lift(s), scale)
    });
    let pen = due(signs::PEN_LEAVES, signs::PEN_LEAVES + signs::PEN_LEAVE).then(|| {
        let (_, turn) = leaving(s, signs::PEN_LEAVES, signs::PEN_LEAVE);
        props.pen(turn, scale)
    });
    [clipboard, pen]
}

/// Draw every frame of the signing, ahead of it.
pub(super) fn warm(props: &Props, scale: f32) {
    let mut last = f32::MIN;
    for s in stage::ticks(0., signs::DONE) {
        draw(props, last, s, scale);
        last = s;
    }
}

impl Onboarding {
    /// Seconds since "Accept and continue" was pressed, at `t` on the
    /// third screen's clock.
    pub fn signing_at(&self, t: f32) -> Option<f32> {
        self.signed.map(|signed| t - signed)
    }

    /// "Accept and continue": the signing plays, then the fourth screen
    /// comes on.
    pub fn accept(&mut self, cx: &mut Context<Self>) {
        let (Some(pressed), Some(t), None) = (self.terms, self.terms_now(), self.signed) else {
            return;
        };
        // Not until the screen has come to rest.
        if t < at::END || self.rewinding().is_some() {
            return;
        }
        self.signed = Some(t);
        self.keep_terms(cx);
        cx.notify();
        // While it signs, the seal's flight is drawn ahead.
        if let Some(seal) = self.seal.clone() {
            let scale = self.scale;
            cx.background_spawn(async move { stage::ahead(|| seal.warm(scale)) })
                .detach();
        }
        cx.spawn(async move |view, cx| {
            let mut last = f32::MIN;
            let mut lead = 1. / 60.;
            loop {
                let Ok(Some((s, at, scale, props))) = view.update(cx, |view, _| {
                    let t = view.terms_now()?;
                    let s = view.signing_at(t)?;
                    (view.terms == Some(pressed)).then(|| (s, t, view.scale, view.props.clone()))
                }) else {
                    return;
                };
                if s >= signs::DONE {
                    break;
                }
                let began = Instant::now();
                let shown = stage::snap(s + lead);
                let drawn_for = at + shown - s;
                let frames = cx
                    .background_spawn(
                        async move { props.map(|props| draw(&props, last, shown, scale)) },
                    )
                    .await;
                last = shown;
                lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                let kept = view.update(cx, |view, cx| {
                    let [clipboard, pen] = frames.unwrap_or_default();
                    for (slot, new) in [(&mut view.clipboard, clipboard), (&mut view.pen, pen)] {
                        if let Some(image) = new {
                            let new = Drawn {
                                image,
                                at: drawn_for,
                            };
                            stage::put(slot, new, cx);
                        }
                    }
                    cx.notify();
                });
                if kept.is_err() {
                    return;
                }
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
            let _ = view.update(cx, |view, cx| view.begin_passkey(cx));
        })
        .detach();
    }

    /// Jump to the end of the signing and on to the fourth screen, as a
    /// test wants it.
    pub fn skip_signing(&mut self, cx: &mut Context<Self>) {
        if self.signed.is_some() {
            self.begin_passkey(cx);
        }
    }

    /// Tell whoever listens that a new account was chosen, once.
    pub(super) fn hand_over(&mut self, cx: &mut Context<Self>) {
        if !std::mem::replace(&mut self.handed_over, true) {
            cx.emit(Chosen::NewAccount);
        }
    }

    /// Where the pen's picture is moved to at `s`, in points from its rest:
    /// its tip on the signature while it writes.
    pub(super) fn pen_reach(&self, s: f32) -> (f32, f32) {
        let Some(props) = &self.props else {
            return (0., 0.);
        };
        let written = signature::written(written(s));
        let on_line = written.last().copied().unwrap_or_default();
        let (x, y) = props.signature_point(on_line);
        let (tip_x, tip_y) = props.pen_tip();
        let (left, top, wide, tall) = design::CLIPBOARD;
        let (pen_left, pen_top, pen_wide, pen_tall) = design::PEN;
        // Both from the window's corner: the point on the page, and the
        // resting tip.
        let page = (left + wide / 2. + x, top + tall / 2. + y);
        let tip = (
            pen_left + pen_wide / 2. + tip_x,
            pen_top + pen_tall / 2. + tip_y,
        );
        let reached = reached(s);
        ((page.0 - tip.0) * reached, (page.1 - tip.1) * reached)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pages_are_up_for_the_whole_of_the_writing() {
        assert!(lift(0.).abs() < 1e-3);
        assert!(lift(signs::LIFT * 0.5) > 0.2);
        assert!((lift(signs::WRITES) - 1.).abs() < 1e-3);
        assert!((lift(signs::RELEASES) - 1.).abs() < 1e-3);
        assert!(lift(signs::CLIPBOARD_LEAVES).abs() < 1e-3);
    }

    #[test]
    fn the_pen_writes_only_while_it_is_on_the_page() {
        assert_eq!(written(signs::WRITES), 0.);
        assert!((reached(signs::WRITES) - 1.).abs() < 1e-3);
        assert_eq!(written(signs::WRITES + signs::WRITE), 1.);
        assert!((reached(signs::PEN_RETURNS) - 1.).abs() < 1e-3);
        assert!(reached(signs::PEN_RETURNS + signs::REACH).abs() < 1e-3);
    }

    #[test]
    fn the_clipboard_leaves_before_the_pen() {
        let (clipboard, turn) = leaving(signs::PEN_LEAVES, signs::CLIPBOARD_LEAVES, signs::LEAVE);
        let (pen, _) = leaving(signs::PEN_LEAVES, signs::PEN_LEAVES, signs::PEN_LEAVE);
        assert!(clipboard > 0.1 && turn > 30. && pen < 1e-3);
    }
}

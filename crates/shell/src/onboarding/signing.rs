// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What "Accept and continue" sets going on the third screen: the
//! clipboard's pages fold over to the last, the pen goes to it and signs
//! "hello", the pages come back down, the clipboard turns over and leaves
//! by the left edge, the pen follows, and onboarding hands over.
//!
//! Its times are seconds since the press, which is itself a time on the
//! third screen's clock, so every drawn frame is still placed by one clock.

use std::sync::Arc;
use std::time::Instant;

use ui::onboarding::terms::{self as design, at, signs};
use ui::{motion, AppContext as _, Context, RenderImage};

use super::props::Props;
use super::stage::Drawn;
use super::{signature, Chosen, Onboarding, FRAME};

/// How far over each folding page is `s` seconds in, in degrees, first
/// page first. They fold from the first, and come back from the last.
pub fn folds(s: f32) -> [f32; design::PAGES] {
    std::array::from_fn(|page| {
        let turn =
            |starts: f32| motion::ease(motion::CARRY, motion::progress(s, starts, signs::PAGE));
        let up = turn(signs::PAGE_STAGGER * page as f32);
        let back = (design::PAGES - 1 - page) as f32;
        let down = turn(signs::UNFOLDS + signs::PAGE_STAGGER * back);
        design::FOLDED * (up - down)
    })
}

/// Is the last page's signature in view? It is while the last page to
/// fold is far enough over.
pub fn uncovered(s: f32) -> bool {
    folds(s)[design::PAGES - 1] > design::UNCOVERED
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
    let folding = due(0., signs::FOLDED);
    let unfolding = due(signs::UNFOLDS, signs::CLIPBOARD_LEAVES);
    let going = due(
        signs::CLIPBOARD_LEAVES,
        signs::CLIPBOARD_LEAVES + signs::LEAVE,
    );
    let clipboard = (folding || unfolding || going).then(|| {
        let (_, turn) = leaving(s, signs::CLIPBOARD_LEAVES, signs::LEAVE);
        props.clipboard(turn, &folds(s), scale)
    });
    let pen = due(signs::PEN_LEAVES, signs::PEN_LEAVES + signs::PEN_LEAVE).then(|| {
        let (_, turn) = leaving(s, signs::PEN_LEAVES, signs::PEN_LEAVE);
        props.pen(turn, scale)
    });
    [clipboard, pen]
}

impl Onboarding {
    /// Seconds since "Accept and continue" was pressed, at `t` on the
    /// third screen's clock.
    pub fn signing_at(&self, t: f32) -> Option<f32> {
        self.signed.map(|signed| t - signed)
    }

    /// "Accept and continue": the signing plays, then onboarding hands
    /// over. Its later screens aren't built.
    pub fn accept(&mut self, cx: &mut Context<Self>) {
        let (Some(pressed), Some(t), None) = (self.terms, self.terms_now(), self.signed) else {
            return;
        };
        // Not until the screen has come to rest.
        if t < at::END {
            return;
        }
        self.signed = Some(t);
        cx.notify();
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
                let (shown, drawn_for) = (s + lead, at + lead);
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
                            if let Some(old) = slot.replace(new) {
                                cx.drop_image(old.image, None);
                            }
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
            let _ = view.update(cx, |view, cx| view.hand_over(cx));
        })
        .detach();
    }

    /// Jump to the end of the signing, as a test wants it.
    pub fn skip_signing(&mut self, cx: &mut Context<Self>) {
        if self.signed.is_some() {
            self.hand_over(cx);
        }
    }

    /// Tell whoever listens that a new account was chosen, once.
    fn hand_over(&mut self, cx: &mut Context<Self>) {
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
    fn the_pages_fold_from_the_first_and_come_back_from_the_last() {
        let early = folds(signs::PAGE * 0.5);
        assert!(early[0] > early[design::PAGES - 1]);
        let folded = folds(signs::FOLDED);
        assert!(folded
            .iter()
            .all(|fold| (fold - design::FOLDED).abs() < 0.5));
        assert!(uncovered(signs::WRITES));
        let coming_back = folds(signs::UNFOLDS + signs::PAGE * 0.5);
        assert!(coming_back[design::PAGES - 1] < coming_back[0]);
        let down = folds(signs::CLIPBOARD_LEAVES);
        assert!(down.iter().all(|fold| fold.abs() < 0.5));
        assert!(!uncovered(signs::CLIPBOARD_LEAVES));
    }

    #[test]
    fn the_pen_writes_only_while_it_is_on_the_page() {
        assert_eq!(written(signs::WRITES), 0.);
        assert!((reached(signs::WRITES) - 1.).abs() < 1e-3);
        assert_eq!(written(signs::WRITES + signs::WRITE), 1.);
        assert!((reached(signs::PEN_RETURNS) - 1.).abs() < 1e-3);
        assert!(reached(signs::UNFOLDS).abs() < 1e-3);
    }

    #[test]
    fn the_clipboard_leaves_before_the_pen() {
        let (clipboard, turn) = leaving(signs::PEN_LEAVES, signs::CLIPBOARD_LEAVES, signs::LEAVE);
        let (pen, _) = leaving(signs::PEN_LEAVES, signs::PEN_LEAVES, signs::PEN_LEAVE);
        assert!(clipboard > 0.1 && turn > 30. && pen < 1e-3);
    }
}

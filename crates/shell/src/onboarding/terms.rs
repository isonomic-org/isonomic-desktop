// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding's third screen, "Before you continue": what moves when
//! "Start new account" is pressed, and what its two buttons do.
//!
//! The two buttons leave, the laptop turns over into the sidebar, the
//! content slides in from the right, the stepper comes up, the clipboard
//! and the pen turn over as they come out from under the content, and the dotted
//! line between the laptop and the clipboard starts to pulse. Like the
//! first two screens, it runs from one clock, which starts at the press,
//! and its 3D frames are drawn off the foreground thread.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ui::onboarding::terms::{self as design, at};
use ui::onboarding::{self as first};
use ui::{motion, AppContext as _, Context, RenderImage};

use super::laptop::{self, Scene};
use super::props::Props;
use super::rewind::Land;
use super::stage::{self, Drawn};
use super::{signing, Onboarding, FRAME};

/// How far the laptop is on its way to the sidebar at `t` (0 to 1), and
/// the turn it has left to make, in degrees.
pub fn laptop_move(t: f32) -> (f32, f32) {
    let through = motion::progress(t, at::LAPTOP_MOVES, at::LAPTOP_MOVE);
    let gone = motion::ease(motion::CARRY, through);
    (gone, (1. - gone) * 360. * design::TURNS)
}

/// The laptop's picture box when it is `gone` of the way to the sidebar.
pub fn laptop_box(gone: f32) -> (f32, f32, f32, f32) {
    let (from, to) = (first::LAPTOP, design::LAPTOP);
    (
        motion::between(from.0, to.0, gone),
        motion::between(from.1, to.1, gone),
        motion::between(from.2, to.2, gone),
        motion::between(from.3, to.3, gone),
    )
}

/// A prop that sets off at `starts`: how far in it is at `t` (0 to 1) and
/// the turn it has left to make, in degrees.
pub fn flight(t: f32, starts: f32) -> (f32, f32) {
    let arrived = motion::ease(motion::CARRY, motion::progress(t, starts, at::FLIGHT));
    (arrived, (1. - arrived) * 360. * design::TURNS)
}

/// Dot `index` of the line at `t`: how solid it is (0 to 1) and its
/// radius. The dots come out in turn; then a pulse runs down them from the
/// laptop to the clipboard, again and again.
pub fn dot(t: f32, index: usize) -> (f32, f32) {
    let k = index as f32;
    let shown = motion::progress(t, at::DOTS_START + k * at::DOT_STAGGER, at::DOT_SHOWS);
    let all_shown = at::DOTS_START + design::DOTS as f32 * at::DOT_STAGGER + at::DOT_SHOWS;
    let mut swell = 0.;
    if t > all_shown {
        let run = design::DOTS as f32 + design::PULSE_LENGTH;
        let head = ((t - all_shown) / at::PULSE).fract() * run;
        let behind = head - k;
        if (0. ..=design::PULSE_LENGTH).contains(&behind) {
            swell = (std::f32::consts::PI * behind / design::PULSE_LENGTH).sin();
        }
    }
    (
        shown * motion::between(design::DOT_REST, 1., swell),
        design::DOT + design::DOT_SWELL * swell,
    )
}

/// The frames due between the last drawn time and `t`. A piece that has
/// landed is drawn once more, at rest and in full detail, and then left.
struct Frames {
    laptop: Option<Arc<RenderImage>>,
    clipboard: Option<Arc<RenderImage>>,
    pen: Option<Arc<RenderImage>>,
}

fn draw(scene: Option<&Scene>, props: Option<&Props>, last: f32, t: f32, scale: f32) -> Frames {
    // Each is first drawn a little ahead of its move.
    let due = |starts: f32, length: f32| t >= starts - 0.05 && last < starts + length;
    let laptop = scene
        .filter(|_| due(at::LAPTOP_MOVES, at::LAPTOP_MOVE))
        .map(|scene| {
            let (gone, turn) = laptop_move(t);
            let (_, _, wide, tall) = laptop_box(gone);
            laptop::posed(scene, turn, 1., (wide, tall), scale)
        });
    let clipboard = props
        .filter(|_| due(at::CLIPBOARD_FLIES, at::FLIGHT))
        .map(|props| {
            let (_, turn) = flight(t, at::CLIPBOARD_FLIES);
            props.clipboard(turn, 0., scale)
        });
    let pen = props
        .filter(|_| due(at::PEN_FLIES, at::FLIGHT))
        .map(|props| props.pen(flight(t, at::PEN_FLIES).1, scale));
    Frames {
        laptop,
        clipboard,
        pen,
    }
}

/// The longest a press on "Start new account" waits for the screen's
/// frames to be drawn ahead.
const WAIT: Duration = Duration::from_millis(600);

/// Draw every frame of the screen's coming on, ahead of it.
pub(super) fn warm(scene: Option<&Scene>, props: Option<&Props>, scale: f32) {
    let mut last = f32::MIN;
    for t in stage::ticks(at::LAPTOP_MOVES - 0.05, at::PEN_FLIES + at::FLIGHT + 0.1) {
        draw(scene, props, last, t, scale);
        last = t;
    }
}

impl Onboarding {
    /// Seconds since "Start new account" was pressed, on the third screen.
    pub fn terms_now(&self) -> Option<f32> {
        let pressed = self.terms?;
        // Going back, the clock runs backward.
        Some(match self.rewinding() {
            Some(rewind) => rewind.now(),
            None => pressed.elapsed().as_secs_f32(),
        })
    }

    /// Jump to the end of the third screen's motion, as a test wants it.
    pub fn skip_terms(&mut self, cx: &mut Context<Self>) {
        let end = Duration::from_secs_f32(at::END + 1.);
        if let Some(pressed) = self.terms.as_mut() {
            *pressed = pressed.checked_sub(end).unwrap_or(*pressed);
        }
        cx.notify();
    }

    /// "Decline": what brought the terms on plays backward, to the two
    /// ways to start.
    pub fn decline(&mut self, cx: &mut Context<Self>) {
        // Once accepted there is no going back.
        if self.signed.is_some() || self.rewinding().is_some() {
            return;
        }
        if let Some(t) = self.terms_now() {
            self.rewind_to((t.min(at::END), 0., 0.), Land::Start, cx);
        }
    }

    /// Declined, and everything is back where it came from: the two ways
    /// to start.
    pub(super) fn declined(&mut self, cx: &mut Context<Self>) {
        self.terms = None;
        for frame in [&mut self.clipboard, &mut self.pen] {
            if let Some(old) = frame.take() {
                cx.drop_image(old.image, None);
            }
        }
        // The laptop is back in the centre: its frame counts as landed
        // there, and is drawn again at that size.
        let landed = first::at::LAPTOP_RISES + first::at::LAPTOP_FLIGHT;
        if let Some(frame) = self.frame.as_mut() {
            frame.at = landed;
        }
        if let Some(scene) = self.scene.clone() {
            let scale = self.scale;
            cx.spawn(async move |view, cx| {
                let image = cx
                    .background_spawn(async move { laptop::draw(&scene, landed, scale) })
                    .await;
                let _ = view.update(cx, |view, cx| {
                    if view.terms.is_some() {
                        return;
                    }
                    let frame = Drawn { image, at: landed };
                    if let Some(old) = view.frame.replace(frame) {
                        cx.drop_image(old.image, None);
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }

    /// Start the third screen's clock and its 3D frames.
    pub(super) fn begin_terms(&mut self, cx: &mut Context<Self>) {
        if self.terms.is_some() || !self.settled() {
            return;
        }
        // Its frames are still being drawn ahead: it starts when they are
        // there, or after a moment if they are slow to come.
        if !self.warmed && !std::mem::replace(&mut self.wants_terms, true) {
            cx.spawn(async move |view, cx| {
                cx.background_executor().timer(WAIT).await;
                let _ = view.update(cx, |view, cx| {
                    if std::mem::take(&mut view.wants_terms) {
                        view.warmed = true;
                        view.begin_terms(cx);
                    }
                });
            })
            .detach();
        }
        if !self.warmed {
            return;
        }
        let pressed = Instant::now();
        self.terms = Some(pressed);
        // The resting laptop's frame is now on this screen's clock, not yet
        // moved.
        if let Some(frame) = self.frame.as_mut() {
            frame.at = 0.;
        }
        cx.notify();
        cx.spawn(async move |view, cx| {
            let mut last = f32::MIN;
            // How long a round of frames takes to draw: they are drawn for
            // that far ahead.
            let mut lead = 1. / 60.;
            loop {
                let Ok(Some((t, scale, scene, props))) = view.update(cx, |view, _| {
                    // Declined, or pressed again since: this run is over.
                    (view.terms == Some(pressed)).then(|| {
                        let t = view.terms_now().unwrap_or(0.);
                        (t, view.scale, view.scene.clone(), view.props.clone())
                    })
                }) else {
                    return;
                };
                let began = Instant::now();
                // Drawn for when it will be on screen.
                let shown_at = stage::snap(t + lead);
                let frames = cx
                    .background_spawn(async move {
                        draw(scene.as_deref(), props.as_deref(), last, shown_at, scale)
                    })
                    .await;
                last = shown_at;
                lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                let kept = view.update(cx, |view, cx| {
                    if view.terms != Some(pressed) {
                        return false;
                    }
                    for (slot, new) in [
                        (&mut view.frame, frames.laptop),
                        (&mut view.clipboard, frames.clipboard),
                        (&mut view.pen, frames.pen),
                    ] {
                        if let Some(image) = new {
                            let new = Drawn {
                                image,
                                at: shown_at,
                            };
                            if let Some(old) = slot.replace(new) {
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
                // The signing draws the props' frames from its press on.
                let signing = view.update(cx, |view, _| view.signed.is_some());
                if t >= at::PEN_FLIES + at::FLIGHT || signing.unwrap_or(true) {
                    // At rest: the signing is drawn ahead.
                    let Ok((Some(props), scale)) =
                        view.update(cx, |view, _| (view.props.clone(), view.scale))
                    else {
                        return;
                    };
                    cx.background_spawn(
                        async move { stage::ahead(|| signing::warm(&props, scale)) },
                    )
                    .detach();
                    break;
                }
                // No faster than the screen shows them.
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_laptop_goes_from_the_centre_to_the_sidebar_and_ends_upright() {
        let (gone, turn) = laptop_move(at::LAPTOP_MOVES);
        assert!(gone.abs() < 1e-3 && (turn - 360.).abs() < 0.5);
        assert_eq!(laptop_box(0.), first::LAPTOP);
        let (gone, turn) = laptop_move(at::LAPTOP_MOVES + at::LAPTOP_MOVE);
        assert!((gone - 1.).abs() < 1e-3 && turn.abs() < 0.5);
        assert_eq!(laptop_box(1.), design::LAPTOP);
    }

    #[test]
    fn the_pulse_runs_from_the_laptop_toward_the_clipboard() {
        // Before the dots start there are none.
        assert_eq!(dot(at::DOTS_START - 0.1, 0).0, 0.);
        // At rest every dot is faint and its own size.
        let all_shown = at::DOTS_START + design::DOTS as f32 * at::DOT_STAGGER + at::DOT_SHOWS;
        let crest = |t: f32| {
            (0..design::DOTS)
                .max_by(|a, b| dot(t, *a).1.total_cmp(&dot(t, *b).1))
                .unwrap()
        };
        // Later in a pulse, the swollen dot is further down the line.
        let early = crest(all_shown + at::PULSE * 0.3);
        let late = crest(all_shown + at::PULSE * 0.7);
        assert!(late > early, "{early} then {late}");
        let (solid, radius) = dot(all_shown + at::PULSE * 0.3, early);
        assert!(solid > design::DOT_REST && radius > design::DOT);
    }
}

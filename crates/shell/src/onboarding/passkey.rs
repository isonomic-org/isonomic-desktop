// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding's fourth screen, "Create your account with Face ID", which
//! follows the signing: its panel slides over the last from the right, and
//! the Face ID seal turns over as it comes out from under it into the
//! sidebar, where rings of light breathe around it.
//!
//! The seal is the devices repository's 3D model. Its rings of light are
//! the model's too, but they are see-through, which `device3d` doesn't
//! draw, so the window draws them, breathing as the model's own animation
//! does.
//!
//! Its times are seconds since the signing ended, which is itself a time
//! on the third screen's clock, so every drawn frame is placed by one
//! clock.

use std::sync::Arc;
use std::time::Instant;

use device3d::{Camera, Framing, Model, ModelError, Transform};
use ui::onboarding::passkey::{self as design, at};
use ui::onboarding::terms;
use ui::{motion, AppContext as _, Context, RenderImage};

use super::rewind::Land;
use super::stage::{self, Display, Drawn};
use super::{Onboarding, FRAME};

static SEAL: &[u8] = include_bytes!("../../../../assets/onboarding/face-id-seal.glb");
/// The model's pieces that are light, not metal.
const LIGHT: [&str; 4] = ["glow_a_fill", "glow_a_line", "glow_b_fill", "glow_b_line"];

pub struct Seal {
    model: Model,
    camera: Camera,
}

impl Seal {
    /// Read the model. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Seal, ModelError> {
        let mut model = Model::read(SEAL)?;
        // The camera's frame still takes in the rings: the wider one is as
        // wide as the frame.
        let light: Vec<_> = LIGHT.iter().filter_map(|name| model.part(name)).collect();
        model.triangles.retain(|triangle| {
            !light
                .iter()
                .any(|part| part.contains(&triangle.vertices[0]))
        });
        // Seen from above, as it lies.
        let camera = Camera::front(
            &model,
            terms::CLIPBOARD_ELEVATION,
            Framing {
                fill: 1.,
                lift: 0.,
                aspect: 1.,
            },
        );
        Ok(Seal { model, camera })
    }

    /// The seal with `turn` degrees still to make: it turns over like a
    /// coin, about the line up the page.
    pub fn draw(&self, turn: f32, scale: f32) -> Arc<RenderImage> {
        let side = design::SEAL_FRAME * design::SEAL_CANVAS;
        stage::frame(
            (&self.model, &self.camera, &[]),
            Transform::turn_about(self.camera.up, turn.to_radians(), self.model.centre()),
            true,
            (side, side),
            design::SEAL_FRAME,
            scale,
            Display::NONE,
        )
    }
}

impl Seal {
    /// Draw every frame of its flight, ahead of it.
    pub fn warm(&self, scale: f32) {
        for p in stage::ticks(at::SEAL_FLIES - 0.05, at::SEAL_FLIES + at::FLIGHT + 0.05) {
            self.draw(flight(p).1, scale);
        }
    }
}

/// How far in the seal is `p` seconds in (0 to 1), and the turn it has
/// left to make, in degrees.
pub fn flight(p: f32) -> (f32, f32) {
    let arrived = motion::ease(
        motion::CARRY,
        motion::progress(p, at::SEAL_FLIES, at::FLIGHT),
    );
    (arrived, (1. - arrived) * 360. * design::TURNS)
}

/// Ring `index` of the light at `p`: the share of its full width it is at,
/// and how solid its fill and its line are. One breath: the nearer ring
/// swells first, the wider one follows.
pub fn ring(p: f32, index: usize) -> (f32, f32, f32) {
    let (_, least, fill, line, kept) = design::RINGS[index];
    let shown = motion::progress(p, at::LIGHT_SHOWS, at::LIGHT_FADE);
    let phase = (p - at::LIGHT_SHOWS).max(0.) / at::BREATH - design::RING_LAG * index as f32;
    let breath = (1. - (std::f32::consts::TAU * phase).cos()) / 2.;
    let solid = shown * (kept + (1. - kept) * breath);
    (least + (1. - least) * breath, fill * solid, line * solid)
}

impl Onboarding {
    /// Seconds since the fourth screen started, at `t` on the third
    /// screen's clock.
    pub fn passkey_at(&self, t: f32) -> Option<f32> {
        self.passkey.map(|started| t - started)
    }

    /// The signing is over: bring the fourth screen on.
    pub(super) fn begin_passkey(&mut self, cx: &mut Context<Self>) {
        let (Some(pressed), Some(started), None) = (self.terms, self.terms_now(), self.passkey)
        else {
            return;
        };
        self.passkey = Some(started);
        cx.notify();
        cx.spawn(async move |view, cx| {
            let mut last = f32::MIN;
            let mut lead = 1. / 60.;
            loop {
                let Ok(Some((p, at_time, scale, seal))) = view.update(cx, |view, _| {
                    let t = view.terms_now()?;
                    let p = view.passkey_at(t)?;
                    (view.terms == Some(pressed)).then(|| (p, t, view.scale, view.seal.clone()))
                }) else {
                    return;
                };
                let ends = at::SEAL_FLIES + at::FLIGHT;
                let began = Instant::now();
                let shown = stage::snap(p + lead);
                let drawn_for = at_time + shown - p;
                let due = shown >= at::SEAL_FLIES - 0.05 && last < ends;
                if let (true, Some(seal)) = (due, seal) {
                    let image = cx
                        .background_spawn(async move { seal.draw(flight(shown).1, scale) })
                        .await;
                    last = shown;
                    lead = began.elapsed().as_secs_f32().clamp(1. / 60., 0.1);
                    let kept = view.update(cx, |view, cx| {
                        if view.passkey.is_none() {
                            return;
                        }
                        let frame = Drawn {
                            image,
                            at: drawn_for,
                        };
                        if let Some(old) = view.seal_frame.replace(frame) {
                            cx.drop_image(old.image, None);
                        }
                        cx.notify();
                    });
                    if kept.is_err() {
                        return;
                    }
                }
                if p >= ends {
                    return;
                }
                if let Some(spare) = FRAME.checked_sub(began.elapsed()) {
                    cx.background_executor().timer(spare).await;
                }
            }
        })
        .detach();
    }

    /// Jump to the end of the fourth screen's motion, as a test wants it.
    pub fn skip_passkey(&mut self, cx: &mut Context<Self>) {
        if let Some(started) = self.passkey.as_mut() {
            *started -= at::END + 1.;
        }
        cx.notify();
    }

    /// "Previous step": the screen's coming on plays backward, the
    /// clipboard and the pen come back, the signing is undone, and the
    /// terms are as they were before they were accepted.
    pub fn previous_step(&mut self, cx: &mut Context<Self>) {
        if self.asking.is_some() || self.flow.is_some() || self.rewinding().is_some() {
            return;
        }
        let (Some(started), Some(signed), Some(t)) = (self.passkey, self.signed, self.terms_now())
        else {
            return;
        };
        let from = t.min(started + at::END);
        let returned = signed + terms::signs::CLIPBOARD_LEAVES;
        self.rewind_to((from, returned, signed), Land::Terms, cx);
    }

    /// The terms are as they were before they were accepted.
    pub(super) fn unsign(&mut self, cx: &mut Context<Self>) {
        let Some(t) = self.terms_now() else {
            return;
        };
        self.passkey = None;
        self.signed = None;
        if let Some(old) = self.seal_frame.take() {
            cx.drop_image(old.image, None);
        }
        // The clipboard and the pen are drawn again where they rested.
        if let Some(props) = self.props.clone() {
            let scale = self.scale;
            cx.spawn(async move |view, cx| {
                let drawn = cx
                    .background_spawn(async move {
                        (props.clipboard(0., 0., scale), props.pen(0., scale))
                    })
                    .await;
                let _ = view.update(cx, |view, cx| {
                    if view.signed.is_some() || view.terms.is_none() {
                        return;
                    }
                    for (slot, image) in [(&mut view.clipboard, drawn.0), (&mut view.pen, drawn.1)]
                    {
                        if let Some(old) = slot.replace(Drawn { image, at: t }) {
                            cx.drop_image(old.image, None);
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seal_loads_and_draws_on_its_canvas() {
        let seal = Seal::load().unwrap();
        assert_eq!(seal.draw(0., 1.).size(0).width.0, 240);
    }

    #[test]
    fn the_seal_arrives_upright_and_then_the_light_breathes() {
        let (arrived, turn) = flight(at::SEAL_FLIES);
        assert!(arrived < 1e-3 && (turn - 360.).abs() < 0.5);
        let (arrived, turn) = flight(at::SEAL_FLIES + at::FLIGHT);
        assert!((arrived - 1.).abs() < 1e-3 && turn.abs() < 0.5);
        // No light before the seal has landed.
        assert_eq!(ring(at::SEAL_FLIES, 0).1, 0.);
        // Half a breath in, the nearer ring is at its widest and most solid.
        let (wide, fill, line) = ring(at::LIGHT_SHOWS + at::BREATH * 0.5, 0);
        assert!((wide - 1.).abs() < 1e-3 && fill > 0.08 && line > 0.15);
        // Between breaths it keeps some of its light; the wider one none.
        let between = at::LIGHT_SHOWS + at::BREATH * 2.;
        assert!(ring(between, 0).1 > 0.02);
        assert!(ring(between + at::BREATH * design::RING_LAG, 1).1 < 1e-3);
    }
}

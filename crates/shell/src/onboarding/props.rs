// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clipboard and the pen of onboarding's third screen: their 3D models
//! from the devices repository, each with the camera it rests under. They
//! turn over on the way in and stay as their last drawn frame.

use std::sync::Arc;

use device3d::{Camera, Framing, Model, ModelError, Transform};
use ui::onboarding::terms as design;
use ui::RenderImage;

use super::stage::{self, Display};

static CLIPBOARD: &[u8] = include_bytes!("../../../../assets/onboarding/clipboard-white.glb");
static PEN: &[u8] = include_bytes!("../../../../assets/onboarding/anud-pen.glb");

pub struct Props {
    clipboard: (Model, Camera),
    pen: (Model, Camera),
}

impl Props {
    /// Read both models. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Props, ModelError> {
        let clipboard = Model::read(CLIPBOARD)?;
        let (_, _, wide, tall) = design::CLIPBOARD;
        // Seen from above, as it lies.
        let above = Camera::front(
            &clipboard,
            design::CLIPBOARD_ELEVATION,
            Framing {
                fill: design::CLIPBOARD_FILL,
                lift: 0.,
                aspect: tall / wide,
            },
        );
        let pen = Model::read(PEN)?;
        // Seen from the side, standing on its tip, in a square.
        let beside = Camera::front(
            &pen,
            0.,
            Framing {
                fill: design::PEN_LENGTH / design::PEN_CANVAS,
                lift: 0.,
                aspect: 1.,
            },
        );
        Ok(Props {
            clipboard: (clipboard, above),
            pen: (pen, beside),
        })
    }

    /// The clipboard with `turn` degrees still to make: it turns over like
    /// a card, about the line up the page.
    pub fn clipboard(&self, turn: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.clipboard;
        let (_, _, wide, tall) = design::CLIPBOARD;
        stage::frame(
            (model, camera),
            Transform::turn_about(camera.up, turn.to_radians(), model.centre()),
            turn,
            (wide * design::PROP_CANVAS, tall * design::PROP_CANVAS),
            wide,
            scale,
            Display::NONE,
        )
    }

    /// The pen with `turn` degrees still to make: it tumbles end over end,
    /// then leans as it lies.
    pub fn pen(&self, turn: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.pen;
        let middle = model.centre();
        let tumble = Transform::turn_about(camera.right, turn.to_radians(), middle);
        let lean = Transform::turn_about(camera.back, design::PEN_LEAN.to_radians(), middle);
        stage::frame(
            (model, camera),
            lean.after(&tumble),
            turn,
            (design::PEN_CANVAS, design::PEN_CANVAS),
            design::PEN_CANVAS,
            scale,
            Display::NONE,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_models_load_and_draw_at_rest() {
        let props = Props::load().unwrap();
        let clipboard = props.clipboard(0., 1.).size(0);
        assert_eq!((clipboard.width.0, clipboard.height.0), (178, 238));
        assert_eq!(props.pen(0., 1.).size(0).width.0, 168);
    }
}

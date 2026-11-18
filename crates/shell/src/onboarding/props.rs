// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clipboard and the pen of onboarding's third screen: their 3D models
//! from the devices repository, each with the camera it rests under. They
//! turn over on the way in and out, and stay as their last drawn frame.
//!
//! The clipboard's pages fold over its top, one at a time, down to the
//! last, which has a line to sign on like the first.

use std::ops::Range;
use std::sync::Arc;

use device3d::{vec3, Camera, Framing, Model, ModelError, Transform, Vec3};
use ui::onboarding::terms as design;
use ui::RenderImage;

use super::stage::{self, Display};

static CLIPBOARD: &[u8] = include_bytes!("../../../../assets/onboarding/clipboard-white.glb");
static PEN: &[u8] = include_bytes!("../../../../assets/onboarding/anud-pen.glb");

/// The clipboard's model, in its own metres: the line its pages turn
/// about, under the clip; how far the first page's printing sits above the
/// last page; and the box on the last page the signature is written in
/// (left, baseline, width, height), just above its line.
const HINGE: Vec3 = vec3(0., 0.0045, -0.1455);
const FIRST_TO_LAST: f32 = -0.0012;
const LAST_PAGE: f32 = 0.00436 + FIRST_TO_LAST;
const SIGN_HERE: (f32, f32, f32, f32) = (-0.076, 0.0962, 0.078, 0.021);
/// What is printed on the first page, and turns over with it.
const PRINTED: [&str; 12] = [
    "clause_1",
    "clause_2",
    "clause_3",
    "clause_4",
    "clause_5",
    "clause_6",
    "rule",
    "signature",
    "signature_label",
    "signature_line",
    "subtitle",
    "title",
];

pub struct Props {
    clipboard: (Model, Camera),
    /// The pages that fold, first page first, each with what is on it.
    pages: Vec<Vec<Range<u32>>>,
    pen: (Model, Camera),
}

impl Props {
    /// Read both models. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Props, ModelError> {
        let mut clipboard = Model::read(CLIPBOARD)?;
        // The last page gets the first page's line to sign on.
        for printed in ["signature_line", "signature_label"] {
            clipboard.copy_part(printed, vec3(0., FIRST_TO_LAST, 0.));
        }
        // The file's sheets are numbered from the bottom: the first page
        // is the highest number, the last page is sheet 1 and stays.
        let pages = (0..design::PAGES)
            .map(|page| {
                let sheet = format!("sheet_{}", design::PAGES + 1 - page);
                let printed = PRINTED.iter().filter(|_| page == 0);
                std::iter::once(sheet.as_str())
                    .chain(printed.copied())
                    .filter_map(|name| clipboard.part(name))
                    .collect()
            })
            .collect();
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
            pages,
            pen: (pen, beside),
        })
    }

    /// The clipboard with `turn` degrees still to make (it turns over like
    /// a card, about the line up the page) and each folding page `folds`
    /// degrees over its top.
    pub fn clipboard(
        &self,
        turn: f32,
        folds: &[f32; design::PAGES],
        scale: f32,
    ) -> Arc<RenderImage> {
        let (model, camera) = &self.clipboard;
        let (_, _, wide, tall) = design::CLIPBOARD;
        let across = vec3(1., 0., 0.);
        let parts: Vec<(Range<u32>, Transform)> = self
            .pages
            .iter()
            .zip(folds)
            .filter(|(_, fold)| **fold != 0.)
            .flat_map(|(page, fold)| {
                // Its foot comes up toward the camera and goes over the top.
                let over = Transform::turn_about(across, -fold.to_radians(), HINGE);
                page.iter().map(move |vertices| (vertices.clone(), over))
            })
            .collect();
        stage::frame(
            (model, camera, &parts),
            Transform::turn_about(camera.up, turn.to_radians(), model.centre()),
            true,
            (
                wide * design::CLIPBOARD_CANVAS,
                tall * design::CLIPBOARD_CANVAS,
            ),
            wide,
            scale,
            Display::NONE,
        )
    }

    /// Where a point of the signature's box (one wide and one tall, from
    /// its bottom left) is on the last page, in points right of and below
    /// the middle of the clipboard's box.
    pub fn signature_point(&self, (x, y): (f32, f32)) -> (f32, f32) {
        let (_, camera) = &self.clipboard;
        let (left, baseline, wide, tall) = SIGN_HERE;
        let (u, v, _) = camera.project(vec3(left + x * wide, LAST_PAGE, baseline - y * tall));
        (u * design::CLIPBOARD.2, -v * design::CLIPBOARD.2)
    }

    fn lean(&self) -> Transform {
        let (model, camera) = &self.pen;
        Transform::turn_about(camera.back, design::PEN_LEAN.to_radians(), model.centre())
    }

    /// Where the resting pen's tip is, in points right of and below the
    /// middle of its square.
    pub fn pen_tip(&self) -> (f32, f32) {
        let (model, camera) = &self.pen;
        let middle = model.centre();
        let tip = self.lean().point(vec3(middle.x, model.min.y, middle.z));
        let (u, v, _) = camera.project(tip);
        (u * design::PEN_CANVAS, -v * design::PEN_CANVAS)
    }

    /// The pen with `turn` degrees still to make: it tumbles end over end,
    /// then leans as it lies.
    pub fn pen(&self, turn: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.pen;
        let tumble = Transform::turn_about(camera.right, turn.to_radians(), model.centre());
        stage::frame(
            (model, camera, &[]),
            self.lean().after(&tumble),
            true,
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
        assert_eq!(props.pages.len(), design::PAGES);
        assert_eq!(props.pages[0].len(), 1 + PRINTED.len());
        let clipboard = props.clipboard(0., &[0.; design::PAGES], 1.).size(0);
        assert_eq!((clipboard.width.0, clipboard.height.0), (356, 475));
        assert_eq!(props.pen(0., 1.).size(0).width.0, 168);
    }

    #[test]
    fn the_signature_is_low_on_the_page_and_the_pen_s_tip_is_at_its_foot() {
        let props = Props::load().unwrap();
        let (_, _, wide, tall) = design::CLIPBOARD;
        let (x, y) = props.signature_point((0., 0.));
        assert!(x < 0. && x > -wide / 2., "{x}");
        assert!(y > 0. && y < tall / 2., "{y}");
        let (x, y) = props.pen_tip();
        assert!(x < 0. && y > design::PEN_LENGTH / 3., "{x}, {y}");
    }
}

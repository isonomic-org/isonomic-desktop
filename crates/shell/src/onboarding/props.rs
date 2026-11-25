// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clipboard and the pen of onboarding's third screen: their 3D models
//! from the devices repository, each with the camera it rests under. They
//! turn over on the way in and out, and stay as their last drawn frame.
//!
//! The clipboard's pages lift by their bottom right corner and curl back
//! toward its top left, far enough to bare that corner of the last page,
//! which has a line to sign on there.

use std::ops::Range;
use std::sync::Arc;

use device3d::{vec3, Camera, Framing, Model, ModelError, Transform, Vec3};
use ui::onboarding::terms as design;
use ui::RenderImage;

use super::stage::{self, Display};

static CLIPBOARD: &[u8] = include_bytes!("../../../../assets/onboarding/clipboard-white.glb");
static PEN: &[u8] = include_bytes!("../../../../assets/onboarding/anud-pen.glb");

/// The clipboard's model, in its own metres. The pages lift from their
/// bottom right corner, curling toward the top left: the corner, the way
/// the curl travels across the page, how far it travels, the radius the
/// paper rolls at, and how far round it rolls, from first lifting to fully
/// lifted, in degrees.
const CORNER: Vec3 = vec3(0.1003, 0., 0.1194);
const TOWARD: Vec3 = vec3(-0.604, 0., -0.797);
const PEEL: f32 = 0.13;
const ROLL: f32 = 0.03;
const OVER: (f32, f32) = (60., 150.);
/// Squares along each side of a page, so that it can bend.
const GRID: u32 = 28;
/// How far the first page's printing sits above the last page, and how far
/// right its signature line is moved to sit in the last page's bottom right
/// corner; the height of the last page's printing; and the box there the
/// signature is written in (left, baseline, width, height), just above the
/// line.
const FIRST_TO_LAST: Vec3 = vec3(0.088, -0.0012, 0.);
const LAST_PAGE: f32 = 0.00436 - 0.0012;
const SIGN_HERE: (f32, f32, f32, f32) = (0.012, 0.0962, 0.074, 0.02);
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
    /// Everything that lifts: the pages above the last, and the printing
    /// on the first.
    lifted: Vec<Range<u32>>,
    pen: (Model, Camera),
}

impl Props {
    /// Read both models. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Props, ModelError> {
        let mut clipboard = Model::read(CLIPBOARD)?;
        // The last page gets the first page's line to sign on, in its
        // bottom right corner.
        for printed in ["signature_line", "signature_label"] {
            clipboard.copy_part(printed, FIRST_TO_LAST);
        }
        // Every page above the last (the file's sheet 1) becomes a sheet of
        // small squares, which can bend.
        let flat: Vec<Range<u32>> = (2..=9)
            .filter_map(|sheet| clipboard.part(&format!("sheet_{sheet}")))
            .collect();
        let mut lifted: Vec<Range<u32>> = flat
            .into_iter()
            .map(|flat| bendable(&mut clipboard, flat))
            .collect();
        lifted.extend(PRINTED.iter().filter_map(|name| clipboard.part(name)));
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
            lifted,
            pen: (pen, beside),
        })
    }

    /// The clipboard with `turn` degrees still to make (it turns over like
    /// a card, about the line up the page) and its pages lifted by `lift`,
    /// from 0 (lying flat) to 1.
    pub fn clipboard(&self, turn: f32, lift: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.clipboard;
        let (_, _, wide, tall) = design::CLIPBOARD;
        let curled;
        let model = if lift > 0. {
            let mut bent = model.clone();
            for vertices in &self.lifted {
                for index in vertices.clone() {
                    let vertex = &mut bent.vertices[index as usize];
                    (vertex.position, vertex.normal) = curl(vertex.position, vertex.normal, lift);
                }
            }
            curled = bent;
            &curled
        } else {
            model
        };
        stage::frame(
            (model, camera, &[]),
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

    /// Is this point of the signature's box bare, with the pages lifted by
    /// `lift`? The lifted pages hang over what is nearest their curl.
    pub fn signature_bare(&self, (x, y): (f32, f32), lift: f32) -> bool {
        let (left, baseline, wide, tall) = SIGN_HERE;
        let on_page = vec3(left + x * wide, 0., baseline - y * tall);
        (on_page - CORNER).dot(TOWARD) < PEEL * lift - ROLL
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

    /// The pen with `turn` degrees still to make: it comes and goes
    /// sideways, so it turns about its upright line, then leans as it lies.
    pub fn pen(&self, turn: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.pen;
        let tumble = Transform::turn_about(camera.up, turn.to_radians(), model.centre());
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

/// Replace a flat page with one made of small squares at its top face:
/// the new page's vertices.
fn bendable(model: &mut Model, flat: Range<u32>) -> Range<u32> {
    let corners = &model.vertices[flat.start as usize..flat.end as usize];
    let (mut low, mut high) = (corners[0].position, corners[0].position);
    for corner in corners {
        (low, high) = (low.min(corner.position), high.max(corner.position));
    }
    let paper = model
        .triangles
        .iter()
        .find(|triangle| flat.contains(&triangle.vertices[0]))
        .map_or(0, |triangle| triangle.material);
    model
        .triangles
        .retain(|triangle| !flat.contains(&triangle.vertices[0]));
    let first = model.vertices.len() as u32;
    let side = GRID + 1;
    for row in 0..side {
        for column in 0..side {
            let (across, down) = (column as f32 / GRID as f32, row as f32 / GRID as f32);
            model.vertices.push(device3d::model::Vertex {
                position: vec3(
                    low.x + (high.x - low.x) * across,
                    high.y,
                    low.z + (high.z - low.z) * down,
                ),
                normal: vec3(0., 1., 0.),
            });
        }
    }
    for row in 0..GRID {
        for column in 0..GRID {
            let corner = first + row * side + column;
            for vertices in [
                [corner, corner + side, corner + 1],
                [corner + 1, corner + side, corner + side + 1],
            ] {
                model.triangles.push(device3d::model::Triangle {
                    vertices,
                    material: paper,
                });
            }
        }
    }
    first..model.vertices.len() as u32
}

/// Where a point of a lifting page, and the way it faces, go when the
/// pages are lifted by `lift`. Paper further from the corner than the curl
/// has travelled lies flat; the rest rolls up off the page and on over,
/// straight, toward the top left.
fn curl(position: Vec3, normal: Vec3, lift: f32) -> (Vec3, Vec3) {
    let up = vec3(0., 1., 0.);
    let past = PEEL * lift - (position - CORNER).dot(TOWARD);
    if past <= 0. {
        return (position, normal);
    }
    let over = (OVER.0 + (OVER.1 - OVER.0) * lift).to_radians();
    let round = (past / ROLL).min(over);
    let straight = past - round * ROLL;
    let (sin, cos) = round.sin_cos();
    // Along the page toward the top left, and up off it.
    let along = past - ROLL * sin - straight * cos;
    let rise = ROLL * (1. - cos) + straight * sin;
    let facing = up * cos + TOWARD * sin;
    // Its height above the last page is kept, the way the paper now faces.
    let above = position.y - LAST_PAGE;
    let flat = vec3(position.x, LAST_PAGE, position.z);
    (
        flat + TOWARD * along + up * rise + facing * above,
        facing * normal.y.signum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_models_load_and_draw_at_rest() {
        let props = Props::load().unwrap();
        assert_eq!(props.lifted.len(), 8 + PRINTED.len());
        let clipboard = props.clipboard(0., 0., 1.).size(0);
        assert_eq!((clipboard.width.0, clipboard.height.0), (178, 238));
        assert_eq!(props.clipboard(0., 1., 1.).size(0).width.0, 178);
        assert_eq!(props.pen(0., 1.).size(0).width.0, 168);
    }

    #[test]
    fn the_signature_is_low_on_the_page_and_the_pen_s_tip_is_at_its_foot() {
        let props = Props::load().unwrap();
        let (_, _, wide, tall) = design::CLIPBOARD;
        let (x, y) = props.signature_point((0., 0.));
        assert!(x > 0. && x < wide / 2., "{x}");
        assert!(y > 0. && y < tall / 2., "{y}");
        // Lifted all the way, the whole box is bare; lying flat, none is.
        for corner in [(0., 0.), (0., 1.), (1., 0.), (1., 1.)] {
            assert!(props.signature_bare(corner, 1.));
            assert!(!props.signature_bare(corner, 0.));
        }
        let (x, y) = props.pen_tip();
        assert!(x < 0. && y > design::PEN_LENGTH / 3., "{x}, {y}");
    }
}

#[cfg(test)]
mod curl_tests {
    use super::*;

    #[test]
    fn the_corner_lifts_and_the_top_stays_down() {
        let up = vec3(0., 1., 0.);
        let corner = vec3(CORNER.x, LAST_PAGE + 0.001, CORNER.z);
        assert_eq!(curl(corner, up, 0.), (corner, up));
        let (lifted, facing) = curl(corner, up, 1.);
        assert!(lifted.y > 0.05 && lifted.x < 0.02 && lifted.z < 0.02);
        // Rolled past upright, it faces down.
        assert!(facing.y < 0.);
        let top = vec3(0., LAST_PAGE + 0.001, -0.14);
        assert_eq!(curl(top, up, 1.), (top, up));
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! One drawn frame of a 3D model, as an image the window can show. Slow:
//! everything here is called off the foreground thread.

use std::sync::Arc;

use device3d::{Camera, Model, Shot, Transform};
use image::{Frame, RgbaImage};
use ui::RenderImage;

/// Samples along each side of a pixel.
const SAMPLES: usize = 2;
/// Degrees left to turn below which a model is drawn in full detail.
const FULL_DETAIL_WITHIN: f32 = 20.;

/// A display's light and the linear colours of its picture's corners.
pub struct Display {
    pub light: f32,
    pub picture: [[f32; 3]; 4],
}

impl Display {
    pub const NONE: Display = Display {
        light: 0.,
        picture: [[0.; 3]; 4],
    };
}

/// Draw `model` posed by `pose` on a canvas of `canvas` points, in which
/// the camera's frame is `frame` points wide, for a screen with `scale`
/// pixels to a point. With `turn` degrees still to make it moves fast, so
/// half the detail is enough and keeps the frames coming.
pub fn frame(
    (model, camera): (&Model, &Camera),
    pose: Transform,
    turn: f32,
    canvas: (f32, f32),
    frame: f32,
    scale: f32,
    display: Display,
) -> Arc<RenderImage> {
    let scale = if turn.abs() > FULL_DETAIL_WITHIN {
        scale * 0.5
    } else {
        scale
    };
    let shot = Shot {
        model,
        camera,
        pose,
        width: (canvas.0 * scale).round().max(1.) as usize,
        height: (canvas.1 * scale).round().max(1.) as usize,
        frame_width: frame * scale,
        samples: SAMPLES,
        light: display.light,
        picture: display.picture,
    };
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let drawn = device3d::render(&shot, threads);
    let pixels = RgbaImage::from_raw(drawn.width as u32, drawn.height as u32, drawn.bgra)
        .expect("a frame holds four bytes a pixel");
    Arc::new(RenderImage::new(vec![Frame::new(pixels)]))
}

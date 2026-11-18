// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! One drawn frame of a 3D model, as an image the window can show. Slow:
//! everything here is called off the foreground thread.

use std::sync::Arc;

use device3d::{Camera, Model, Shot, Transform};
use image::{Frame, RgbaImage};
use ui::RenderImage;

/// A drawn frame and the time on its screen's clock it was drawn for.
/// It is placed by that time, so where it is and how far it has turned
/// always agree.
pub struct Drawn {
    pub image: Arc<RenderImage>,
    pub at: f32,
}

/// Samples along each side of a pixel.
const SAMPLES: usize = 2;

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
/// pixels to a point, in full detail or, to keep a large model's frames
/// coming while it moves, in half.
pub fn frame(
    (model, camera): (&Model, &Camera),
    pose: Transform,
    full: bool,
    canvas: (f32, f32),
    frame: f32,
    scale: f32,
    display: Display,
) -> Arc<RenderImage> {
    let scale = if full { scale } else { scale * 0.5 };
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

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The laptop on the onboarding window: its 3D model, turned and drawn a
//! frame at a time off the foreground thread, and the finished picture it
//! lands on.

use std::sync::Arc;

use device3d::{Camera, Framing, Model, Shot, Transform};
use image::{Frame, RgbaImage};
use ui::onboarding::{self as design, at};
use ui::{motion, Image, ImageFormat, RenderImage};

static MODEL: &[u8] = include_bytes!("../../../../assets/onboarding/macbook-pro-13-2020.glb");
static PICTURE: &[u8] = include_bytes!("../../../../assets/onboarding/macbook-pro-13-2020.png");

/// The model's silver, as its colour table names it.
const SILVER: [u8; 3] = [0xCD, 0xCE, 0xD1];
/// The lit display's corners in the finished picture: top left, top right,
/// bottom left, bottom right.
const DISPLAY: [[u8; 3]; 4] = [
    [197, 201, 211],
    [116, 139, 196],
    [190, 150, 205],
    [207, 175, 148],
];
/// Samples along each side of a pixel while the laptop moves.
const SAMPLES: usize = 2;
/// Degrees left to turn below which the laptop is drawn in full detail.
const FULL_DETAIL_WITHIN: f32 = 20.;

/// The model and the camera the finished picture was taken with.
pub struct Scene {
    model: Model,
    camera: Camera,
}

impl Scene {
    /// Read the model. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Scene, device3d::ModelError> {
        let mut model = Model::read(MODEL)?;
        model.recolour("body", SILVER);
        let camera = Camera::front(&model, Framing::LAPTOP_ELEVATION, Framing::LAPTOP);
        Ok(Scene { model, camera })
    }
}

/// The finished picture.
pub fn picture() -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, PICTURE.to_vec()))
}

/// Where the flight is at `now` seconds: how far up (0 at the bottom edge,
/// 1 landed), the turn left to make in degrees, and the display's light.
pub fn flight(now: f32) -> (f32, f32, f32) {
    let through = motion::progress(now, at::LAPTOP_RISES, at::LAPTOP_FLIGHT);
    let landed = motion::ease(motion::GLIDE, through);
    let lit = motion::progress(through, 1. - at::LIGHT_SHARE, at::LIGHT_SHARE);
    (
        landed,
        (1. - landed) * 360. * design::LAPTOP_TURNS,
        lit * lit * (3. - 2. * lit),
    )
}

/// Draw the laptop as it is at `now`, for a screen with `scale` pixels to
/// a point. Slow: call it off the foreground thread.
pub fn draw(scene: &Scene, now: f32, scale: f32) -> Arc<RenderImage> {
    let (_, turn, light) = flight(now);
    let (_, _, wide, tall) = design::LAPTOP;
    // While it turns fast, half the detail is enough and keeps the frames
    // coming; it is drawn in full as it comes to rest.
    let scale = if turn > FULL_DETAIL_WITHIN {
        scale * 0.5
    } else {
        scale
    };
    let frame_width = wide * scale;
    let linear = |c: [u8; 3]| c.map(|c| device3d::shade::to_linear(c as f32 / 255.));
    let shot = Shot {
        model: &scene.model,
        camera: &scene.camera,
        // A somersault: about the camera's level axis, through its middle.
        pose: Transform::turn_about(scene.camera.right, -turn.to_radians(), scene.model.centre()),
        width: (frame_width * design::LAPTOP_CANVAS).round() as usize,
        height: (tall * scale * design::LAPTOP_CANVAS).round() as usize,
        frame_width,
        samples: SAMPLES,
        light,
        picture: DISPLAY.map(linear),
    };
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let frame = device3d::render(&shot, threads);
    let pixels = RgbaImage::from_raw(frame.width as u32, frame.height as u32, frame.bgra)
        .expect("a frame holds four bytes a pixel");
    Arc::new(RenderImage::new(vec![Frame::new(pixels)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_and_its_camera_load() {
        assert!(Scene::load().is_ok());
    }

    #[test]
    fn the_flight_starts_at_the_edge_dark_and_lands_lit_and_upright() {
        let (up, turn, light) = flight(at::LAPTOP_RISES);
        assert!(up.abs() < 1e-3 && light == 0.);
        assert!((turn - 360.).abs() < 1e-3);
        let (up, turn, light) = flight(at::LAPTOP_RISES + at::LAPTOP_FLIGHT);
        assert!((up - 1.).abs() < 1e-3 && turn.abs() < 0.5);
        assert!((light - 1.).abs() < 1e-3);
    }

    #[test]
    fn a_frame_is_the_canvas_at_the_screen_s_scale() {
        let scene = Scene::load().unwrap();
        let landed = draw(&scene, at::LAPTOP_RISES + at::LAPTOP_FLIGHT, 1.);
        let size = landed.size(0);
        assert_eq!((size.width.0, size.height.0), (683, 512));
        // Mid-turn it is drawn at half the detail.
        let turning = draw(&scene, at::LAPTOP_RISES + 0.1, 1.);
        assert_eq!(turning.size(0).width.0, 341);
    }
}

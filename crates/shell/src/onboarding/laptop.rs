// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The laptop on the onboarding window: its 3D model, turned and drawn a
//! frame at a time off the foreground thread, and the finished picture it
//! lands on.

use std::sync::Arc;

use device3d::{Camera, Framing, Model, Transform};
use ui::onboarding::{self as design, at};
use ui::{motion, Image, ImageFormat, RenderImage};

use super::stage::{self, Display};

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
    posed(scene, turn, light, (wide, tall), scale)
}

/// Draw the laptop with `turn` degrees of a somersault still to make, its
/// display lit by `light`, for a picture of `size` points.
pub fn posed(
    scene: &Scene,
    turn: f32,
    light: f32,
    (wide, tall): (f32, f32),
    scale: f32,
) -> Arc<RenderImage> {
    let linear = |c: [u8; 3]| c.map(|c| device3d::shade::to_linear(c as f32 / 255.));
    stage::frame(
        (&scene.model, &scene.camera),
        // A somersault: about the camera's level axis, through its middle.
        Transform::turn_about(scene.camera.right, -turn.to_radians(), scene.model.centre()),
        turn,
        (wide * design::LAPTOP_CANVAS, tall * design::LAPTOP_CANVAS),
        wide,
        scale,
        Display {
            light,
            picture: DISPLAY.map(linear),
        },
    )
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

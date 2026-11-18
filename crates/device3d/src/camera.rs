// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stills' camera: it stands in front of the device, a little above
//! it, looks at its middle, and is zoomed and shifted (never turned)
//! until the model fills a set share of the frame. Because this is how the
//! published stills were framed, a frame drawn here lines up with them.

use crate::math::{vec3, Vec3};
use crate::model::Model;

/// Distance from the model's middle, in model diagonals, and the starting
/// zoom (focal length over sensor width).
const DISTANCE_DIAGONALS: f32 = 4.5;
const BASE_ZOOM: f32 = 85. / 36.;

/// How the model sits in the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Framing {
    /// Share of the frame's width or height the model fills, whichever
    /// binds first.
    pub fill: f32,
    /// Raise the model by this share of the frame's width.
    pub lift: f32,
    /// The frame's height over its width.
    pub aspect: f32,
}

impl Framing {
    /// The laptop stills: 4:3, filled to 84 %, seen from 12° up.
    pub const LAPTOP: Framing = Framing {
        fill: 0.84,
        lift: 0.,
        aspect: 0.75,
    };
    pub const LAPTOP_ELEVATION: f32 = 12.;
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    /// From the model toward the camera.
    pub back: Vec3,
    zoom: f32,
    shift: (f32, f32),
}

impl Camera {
    /// The camera straight in front of `model`, `elevation` degrees above
    /// its horizon.
    pub fn front(model: &Model, elevation: f32, framing: Framing) -> Camera {
        let (sin, cos) = elevation.to_radians().sin_cos();
        // The model's front faces +z, and y is up.
        let back = vec3(0., sin, cos);
        let diagonal = (model.max - model.min).length();
        let mut camera = Camera {
            position: model.centre() + back * (diagonal * DISTANCE_DIAGONALS),
            right: vec3(1., 0., 0.),
            up: vec3(0., cos, -sin),
            back,
            zoom: BASE_ZOOM,
            shift: (0., 0.),
        };
        let (mut u0, mut u1, mut v0, mut v1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for vertex in &model.vertices {
            let (u, v, _) = camera.project(vertex.position);
            (u0, u1) = (u0.min(u), u1.max(u));
            (v0, v1) = (v0.min(v), v1.max(v));
        }
        let k = (framing.fill / (u1 - u0).max(1e-6))
            .min(framing.fill * framing.aspect / (v1 - v0).max(1e-6));
        camera.zoom *= k;
        camera.shift = (k * (u0 + u1) / 2., k * (v0 + v1) / 2. - framing.lift);
        camera
    }

    /// A point in the camera's space: right, up, and toward the camera.
    pub fn view(&self, p: Vec3) -> Vec3 {
        let d = p - self.position;
        vec3(d.dot(self.right), d.dot(self.up), d.dot(self.back))
    }

    /// A direction in the camera's space.
    pub fn view_direction(&self, d: Vec3) -> Vec3 {
        vec3(d.dot(self.right), d.dot(self.up), d.dot(self.back))
    }

    /// Where a point lands in the frame, in frame widths from its centre
    /// (right and up positive), and its distance in front of the camera.
    pub fn project(&self, p: Vec3) -> (f32, f32, f32) {
        let q = self.view(p);
        let depth = (-q.z).max(1e-6);
        (
            self.zoom * q.x / depth - self.shift.0,
            self.zoom * q.y / depth - self.shift.1,
            depth,
        )
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! A device's 3D model drawn into pixels on the processor: read the model
//! file, stand the hero camera in front of it, turn it, and ask for a
//! frame. The window's interface library draws no 3D, so a turning device
//! is a sequence of these frames.
//!
//! Nothing here knows the window: a frame is plain bytes.

pub mod camera;
pub mod math;
pub mod model;
pub mod render;
pub mod shade;

pub use camera::{Camera, Framing};
pub use math::{vec3, Transform, Vec3};
pub use model::{Model, ModelError, Part};
pub use render::{render, Frame, Shot};

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a surface looks: a white studio that stands still around the
//! camera while the device turns in it. No light is traced; a surface
//! answers the room by the way it faces and what it would mirror.

use crate::math::{vec3, Vec3};
use crate::model::Material;

/// Brightness of the finished picture.
const EXPOSURE: f32 = 1.42;
/// The big soft light: up, in front, a little to the left.
const KEY_UNIT: Vec3 = vec3(-0.349_6, 0.719_2, 0.599_3);

pub fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn to_srgb(c: f32) -> f32 {
    let c = c.clamp(0., 1.);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1. / 2.4) - 0.055
    }
}

/// The colour curve as a table: a linear value in 0..=1 to its byte.
pub fn srgb_bytes() -> &'static [u8; 4097] {
    static TABLE: std::sync::OnceLock<[u8; 4097]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u8; 4097];
        for (i, byte) in table.iter_mut().enumerate() {
            *byte = (to_srgb(i as f32 / 4096.) * 255. + 0.5) as u8;
        }
        table
    })
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

/// What a mirror facing `r` would show, blurred by `roughness`.
fn room(r: Vec3, roughness: f32) -> f32 {
    // Brighter overhead than underfoot; rough surfaces see the average.
    let height = smooth(r.y * 0.5 + 0.5);
    let sharp = 0.50 + 0.50 * height;
    let wall = sharp + (0.80 - sharp) * roughness.min(1.);
    let spread = 0.03 + roughness * roughness * 0.9;
    let toward = (r.dot(KEY_UNIT) - 1.) / spread;
    if toward < -7. {
        return wall;
    }
    wall + 0.55 * toward.exp() * (1. - 0.5 * roughness)
}

/// The room's light falling on a matte surface facing `n`.
fn daylight(n: Vec3) -> f32 {
    0.62 + 0.38 * smooth(n.y * 0.5 + 0.5)
}

/// The linear colour of a surface facing `normal` (in the camera's space,
/// the camera looking down its own -z).
pub fn surface(material: &Material, normal: Vec3) -> [f32; 3] {
    let view = vec3(0., 0., 1.);
    let n = if normal.z < 0. { -normal } else { normal };
    let facing = n.dot(view).clamp(0., 1.);
    let mirrored = n * (2. * facing) - view;
    let glance = (1. - facing).powi(5);
    // Polished dark glass and plastic mirror the dim side of the room,
    // not its lights, so they stay near black as they do in the stills.
    let dim = 0.05 + 0.95 * smooth((material.roughness - 0.08) / 0.3).max(material.metallic);
    let seen = room(mirrored, material.roughness) * dim;
    let lit = daylight(n);
    let mut out = [0.; 3];
    for (channel, out) in out.iter_mut().enumerate() {
        let base = material.base[channel];
        let f0 = 0.04 * material.specular * (1. - material.metallic) + base * material.metallic;
        // Rough surfaces lose their grazing mirror.
        let grazing = (1. - material.roughness).max(0.) * (0.3 + 0.7 * material.metallic);
        let fresnel = f0 + (1. - f0) * glance * grazing;
        let matte = base * (1. - material.metallic) * lit;
        *out = (matte * (1. - fresnel) + fresnel * seen) * EXPOSURE;
    }
    out
}

/// A lit display: its picture, through glass that still mirrors the room
/// a little. `light` runs from 0 (off) to 1 (on).
pub fn display(
    material: &Material,
    normal: Vec3,
    corners: &[[f32; 3]; 4],
    u: f32,
    v: f32,
    light: f32,
) -> [f32; 3] {
    let off = surface(material, normal);
    let (u, v) = (u.clamp(0., 1.), v.clamp(0., 1.));
    let mut out = [0.; 3];
    for (channel, out) in out.iter_mut().enumerate() {
        let top = corners[0][channel] + (corners[1][channel] - corners[0][channel]) * u;
        let bottom = corners[2][channel] + (corners[3][channel] - corners[2][channel]) * u;
        let picture = top + (bottom - top) * v;
        *out = off[channel] * (1. - 0.6 * light) + picture * light;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_colour_curve_round_trips() {
        for c in [0., 0.02, 0.2, 0.5, 1.] {
            assert!((to_srgb(to_linear(c)) - c).abs() < 1e-4);
        }
    }

    #[test]
    fn a_display_goes_from_its_dark_glass_to_its_picture() {
        let glass = Material {
            name: "screen_off".into(),
            base: [0.006; 3],
            metallic: 0.,
            roughness: 0.05,
            specular: 1.,
        };
        let white = [[1.; 3]; 4];
        let n = vec3(0., 0., 1.);
        let off = display(&glass, n, &white, 0.5, 0.5, 0.);
        let on = display(&glass, n, &white, 0.5, 0.5, 1.);
        assert!(off[0] < 0.15, "{off:?}");
        assert!(on[0] > 0.95, "{on:?}");
    }
}

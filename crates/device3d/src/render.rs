// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame: every triangle of the posed model, projected, filled with a
//! depth test at several samples a pixel, shaded, and averaged down into
//! pixels with soft edges. The picture is cut into bands, one a thread.

use std::ops::Range;

use crate::camera::Camera;
use crate::math::{Transform, Vec3};
use crate::model::Model;
use crate::shade;

/// What to draw.
pub struct Shot<'a> {
    pub model: &'a Model,
    pub camera: &'a Camera,
    /// Moves the model before the camera sees it.
    pub pose: Transform,
    /// Pieces moved on their own first, each a run of the model's
    /// vertices: a page turned over, say.
    pub parts: &'a [(Range<u32>, Transform)],
    /// The picture's size in pixels.
    pub width: usize,
    pub height: usize,
    /// How many pixels wide the camera's frame is. The frame is centred in
    /// the picture, which may be larger, so a turning model isn't cut off.
    pub frame_width: f32,
    /// Samples along each side of a pixel.
    pub samples: usize,
    /// The display's light, 0 (off) to 1 (on), and the linear colours of
    /// its picture's corners: top left, top right, bottom left, bottom
    /// right.
    pub light: f32,
    pub picture: [[f32; 3]; 4],
}

/// Pixels, top row first: blue, green, red, alpha, not multiplied by alpha.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub bgra: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Corner {
    x: f32,
    y: f32,
    depth: f32,
    normal: Vec3,
    u: f32,
    v: f32,
}

/// What the nearest triangle left at one sample, packed small: a frame
/// holds millions of these.
#[derive(Clone, Copy, PartialEq)]
struct Sample {
    depth: f32,
    /// The surface's direction, each part scaled to an `i16`.
    normal: [i16; 3],
    /// Where the sample sits across the display, scaled to a `u16`.
    u: u16,
    v: u16,
    material: u16,
}

const EMPTY: Sample = Sample {
    depth: f32::MAX,
    normal: [0, 0, i16::MAX],
    u: 0,
    v: 0,
    material: u16::MAX,
};

fn pack(normal: Vec3) -> [i16; 3] {
    let n = normal.normalized();
    [n.x, n.y, n.z].map(|c| (c * 32767.) as i16)
}

fn unit(c: f32) -> u16 {
    (c.clamp(0., 1.) * 65535.) as u16
}

fn corners(shot: &Shot<'_>) -> Vec<Corner> {
    let s = shot.samples as f32;
    let scale = shot.frame_width * s;
    let (cx, cy) = (shot.width as f32 * s / 2., shot.height as f32 * s / 2.);
    let screen = shot.model.screen;
    shot.model
        .vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let own = shot
                .parts
                .iter()
                .find(|(vertices, _)| vertices.contains(&(index as u32)));
            let (position, normal) = match own {
                Some((_, moved)) => (moved.point(vertex.position), moved.direction(vertex.normal)),
                None => (vertex.position, vertex.normal),
            };
            let (u, v, depth) = shot.camera.project(shot.pose.point(position));
            let normal = shot.camera.view_direction(shot.pose.direction(normal));
            // Where the vertex sits across the display, for its picture.
            let (su, sv) = screen.map_or((0., 0.), |screen| {
                let from = vertex.position - screen.centre;
                (
                    0.5 + from.dot(screen.right) / (2. * screen.half_width),
                    0.5 - from.dot(screen.up) / (2. * screen.half_height),
                )
            });
            Corner {
                x: cx + u * scale,
                y: cy - v * scale,
                depth,
                normal,
                u: su,
                v: sv,
            }
        })
        .collect()
}

/// Fill the samples of rows `top..bottom` (in samples) with the nearest
/// triangle at each.
/// The columns of the picture the model can touch, in pixels: the first,
/// and how many. Nothing outside them is filled or shaded.
#[derive(Clone, Copy)]
struct Columns {
    first: usize,
    count: usize,
}

fn fill_band(
    shot: &Shot<'_>,
    corners: &[Corner],
    (top, bottom): (usize, usize),
    columns: Columns,
    out: &mut [Sample],
) {
    let first = (columns.first * shot.samples) as isize;
    let wide = columns.count * shot.samples;
    for triangle in &shot.model.triangles {
        let [a, b, c] = triangle.vertices.map(|i| corners[i as usize]);
        let min_y = a.y.min(b.y).min(c.y);
        let max_y = a.y.max(b.y).max(c.y);
        if max_y < top as f32 || min_y >= bottom as f32 {
            continue;
        }
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area.abs() < 1e-9 {
            continue;
        }
        let y0 = (min_y.floor().max(top as f32)) as usize;
        let y1 = (max_y.ceil() as isize).min(bottom as isize - 1);
        if y1 < y0 as isize {
            continue;
        }
        let inverse = 1. / area;
        let edges = [(a, b), (b, c), (c, a)];
        for y in y0..=y1 as usize {
            let py = y as f32 + 0.5;
            // The row's stretch inside the triangle: where its edges cross
            // the row. A sliver then costs its own samples, not its box's.
            let (mut left, mut right) = (f32::MAX, f32::MIN);
            for (p, q) in edges {
                if (p.y <= py) != (q.y <= py) {
                    let x = p.x + (py - p.y) * (q.x - p.x) / (q.y - p.y);
                    left = left.min(x);
                    right = right.max(x);
                }
            }
            if left > right {
                continue;
            }
            let x0 = ((left - 0.5).ceil() as isize).max(first);
            let x1 = ((right - 0.5).floor() as isize).min(first + wide as isize - 1);
            let row = (y - top) * wide;
            for x in x0..=x1 {
                let px = x as f32 + 0.5;
                let wa = (((b.x - px) * (c.y - py) - (b.y - py) * (c.x - px)) * inverse).max(0.);
                let wb = (((c.x - px) * (a.y - py) - (c.y - py) * (a.x - px)) * inverse).max(0.);
                let wc = (1. - wa - wb).max(0.);
                let depth = wa * a.depth + wb * b.depth + wc * c.depth;
                let sample = &mut out[row + (x - first) as usize];
                if depth >= sample.depth {
                    continue;
                }
                *sample = Sample {
                    depth,
                    normal: pack(a.normal * wa + b.normal * wb + c.normal * wc),
                    u: unit(wa * a.u + wb * b.u + wc * c.u),
                    v: unit(wa * a.v + wb * b.v + wc * c.v),
                    material: triangle.material,
                };
            }
        }
    }
}

/// Shade a band's samples and average them into its pixels.
fn resolve_band(
    shot: &Shot<'_>,
    samples: &[Sample],
    rows: usize,
    columns: Columns,
    out: &mut [u8],
) {
    let s = shot.samples;
    let wide = columns.count * s;
    let screen = shot.model.screen.map(|screen| screen.material);
    let per_pixel = (s * s) as f32;
    let curve = shade::srgb_bytes();
    // Neighbouring samples on a flat face are the same surface: shade once.
    let mut last = (EMPTY, [0.; 3]);
    for y in 0..rows {
        for x in 0..columns.count {
            let (mut sum, mut covered) = ([0.; 3], 0.);
            for sy in 0..s {
                for sx in 0..s {
                    let sample = &samples[(y * s + sy) * wide + x * s + sx];
                    let Some(material) = shot.model.materials.get(sample.material as usize) else {
                        continue;
                    };
                    let [nx, ny, nz] = sample.normal.map(|c| c as f32 / 32767.);
                    let normal = Vec3 {
                        x: nx,
                        y: ny,
                        z: nz,
                    };
                    let lit = Some(sample.material) == screen && shot.light > 0.;
                    let same = !lit
                        && last.0.material == sample.material
                        && last.0.normal == sample.normal;
                    let colour = if same {
                        last.1
                    } else if lit {
                        shade::display(
                            material,
                            normal,
                            &shot.picture,
                            sample.u as f32 / 65535.,
                            sample.v as f32 / 65535.,
                            shot.light,
                        )
                    } else {
                        shade::surface(material, normal)
                    };
                    if !lit {
                        last = (*sample, colour);
                    }
                    for channel in 0..3 {
                        sum[channel] += colour[channel].min(1.);
                    }
                    covered += 1.;
                }
            }
            if covered == 0. {
                continue;
            }
            let pixel = &mut out[(y * shot.width + columns.first + x) * 4..][..4];
            let byte = |c: f32| curve[((c / covered).clamp(0., 1.) * 4096.) as usize];
            pixel.copy_from_slice(&[
                byte(sum[2]),
                byte(sum[1]),
                byte(sum[0]),
                (covered / per_pixel * 255. + 0.5) as u8,
            ]);
        }
    }
}

/// Draw one frame, on `threads` threads.
pub fn render(shot: &Shot<'_>, threads: usize) -> Frame {
    let mut bgra = vec![0u8; shot.width * shot.height * 4];
    if shot.width == 0 || shot.height == 0 || shot.samples == 0 {
        return Frame {
            width: shot.width,
            height: shot.height,
            bgra,
        };
    }
    let corners = corners(shot);
    // The box of pixels the model lands in; the rest stays clear.
    let s = shot.samples as f32;
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for corner in &corners {
        (x0, x1) = (x0.min(corner.x), x1.max(corner.x));
        (y0, y1) = (y0.min(corner.y), y1.max(corner.y));
    }
    let clamp = |v: f32, most: usize| (v.max(0.) as usize).min(most);
    let (left, right) = (
        clamp((x0 / s).floor(), shot.width),
        clamp((x1 / s).ceil() + 1., shot.width),
    );
    let (top, bottom) = (
        clamp((y0 / s).floor(), shot.height),
        clamp((y1 / s).ceil() + 1., shot.height),
    );
    if right <= left || bottom <= top {
        return Frame {
            width: shot.width,
            height: shot.height,
            bgra,
        };
    }
    let columns = Columns {
        first: left,
        count: right - left,
    };
    let rows_each = (bottom - top).div_ceil(threads.max(1)).max(1);
    let corners = &corners;
    let row_bytes = shot.width * 4;
    std::thread::scope(|scope| {
        let touched = &mut bgra[top * row_bytes..bottom * row_bytes];
        for (band, out) in touched.chunks_mut(rows_each * row_bytes).enumerate() {
            scope.spawn(move || {
                let rows = out.len() / row_bytes;
                let first = (top + band * rows_each) * shot.samples;
                let last = first + rows * shot.samples;
                let mut samples = vec![EMPTY; rows * shot.samples * columns.count * shot.samples];
                fill_band(shot, corners, (first, last), columns, &mut samples);
                resolve_band(shot, &samples, rows, columns, out);
            });
        }
    });
    Frame {
        width: shot.width,
        height: shot.height,
        bgra,
    }
}

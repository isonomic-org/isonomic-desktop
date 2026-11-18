// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The little vector algebra the renderer needs.

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Vec3 {
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Vec3) -> Vec3 {
        vec3(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Vec3 {
        let length = self.length();
        if length > 0. {
            self * (1. / length)
        } else {
            self
        }
    }

    pub fn min(self, o: Vec3) -> Vec3 {
        vec3(self.x.min(o.x), self.y.min(o.y), self.z.min(o.z))
    }

    pub fn max(self, o: Vec3) -> Vec3 {
        vec3(self.x.max(o.x), self.y.max(o.y), self.z.max(o.z))
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        vec3(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        vec3(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        vec3(self.x * k, self.y * k, self.z * k)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        vec3(-self.x, -self.y, -self.z)
    }
}

/// A rotation and a translation: `rows` are the rotated axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub rows: [Vec3; 3],
    pub shift: Vec3,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        rows: [vec3(1., 0., 0.), vec3(0., 1., 0.), vec3(0., 0., 1.)],
        shift: vec3(0., 0., 0.),
    };

    /// From a model file's unit quaternion `[x, y, z, w]` and translation.
    pub fn from_parts(q: [f32; 4], t: [f32; 3]) -> Transform {
        let [x, y, z, w] = q;
        Transform {
            rows: [
                vec3(
                    1. - 2. * (y * y + z * z),
                    2. * (x * y - z * w),
                    2. * (x * z + y * w),
                ),
                vec3(
                    2. * (x * y + z * w),
                    1. - 2. * (x * x + z * z),
                    2. * (y * z - x * w),
                ),
                vec3(
                    2. * (x * z - y * w),
                    2. * (y * z + x * w),
                    1. - 2. * (x * x + y * y),
                ),
            ],
            shift: vec3(t[0], t[1], t[2]),
        }
    }

    /// A turn of `radians` about the unit `axis` through `pivot`.
    pub fn turn_about(axis: Vec3, radians: f32, pivot: Vec3) -> Transform {
        let (s, c) = radians.sin_cos();
        let k = 1. - c;
        let Vec3 { x, y, z } = axis;
        let rows = [
            vec3(c + x * x * k, x * y * k - z * s, x * z * k + y * s),
            vec3(y * x * k + z * s, c + y * y * k, y * z * k - x * s),
            vec3(z * x * k - y * s, z * y * k + x * s, c + z * z * k),
        ];
        let turned = Transform {
            rows,
            shift: Vec3::default(),
        };
        Transform {
            rows,
            shift: pivot - turned.direction(pivot),
        }
    }

    /// Rotate a direction.
    pub fn direction(&self, v: Vec3) -> Vec3 {
        vec3(
            self.rows[0].dot(v),
            self.rows[1].dot(v),
            self.rows[2].dot(v),
        )
    }

    /// Move a point.
    pub fn point(&self, p: Vec3) -> Vec3 {
        self.direction(p) + self.shift
    }

    /// `self` after `first`.
    pub fn after(&self, first: &Transform) -> Transform {
        let column = |i: usize| {
            let axis = match i {
                0 => vec3(first.rows[0].x, first.rows[1].x, first.rows[2].x),
                1 => vec3(first.rows[0].y, first.rows[1].y, first.rows[2].y),
                _ => vec3(first.rows[0].z, first.rows[1].z, first.rows[2].z),
            };
            self.direction(axis)
        };
        let (a, b, c) = (column(0), column(1), column(2));
        Transform {
            rows: [
                vec3(a.x, b.x, c.x),
                vec3(a.y, b.y, c.y),
                vec3(a.z, b.z, c.z),
            ],
            shift: self.point(first.shift),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn a_quarter_turn_about_x_takes_y_to_z() {
        let turn = Transform::turn_about(
            vec3(1., 0., 0.),
            std::f32::consts::FRAC_PI_2,
            Vec3::default(),
        );
        assert!(close(turn.direction(vec3(0., 1., 0.)), vec3(0., 0., 1.)));
    }

    #[test]
    fn a_turn_leaves_its_pivot_where_it_is() {
        let pivot = vec3(0.2, -0.1, 0.4);
        let turn = Transform::turn_about(vec3(0., 0., 1.), 1.3, pivot);
        assert!(close(turn.point(pivot), pivot));
    }

    #[test]
    fn a_quaternion_matches_the_same_turn() {
        let half = 0.4_f32;
        let q = Transform::from_parts([half.sin(), 0., 0., half.cos()], [0., 0., 0.]);
        let turn = Transform::turn_about(vec3(1., 0., 0.), 2. * half, Vec3::default());
        let v = vec3(0.3, 0.5, -0.2);
        assert!(close(q.direction(v), turn.direction(v)));
    }

    #[test]
    fn composing_applies_the_first_then_the_second() {
        let first = Transform::from_parts([0., 0., 0., 1.], [1., 0., 0.]);
        let second = Transform::turn_about(
            vec3(0., 0., 1.),
            std::f32::consts::FRAC_PI_2,
            Vec3::default(),
        );
        let both = second.after(&first);
        assert!(close(both.point(Vec3::default()), vec3(0., 1., 0.)));
    }
}

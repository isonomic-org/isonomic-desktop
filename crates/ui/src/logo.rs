// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Isonomic mark and its intro, "Converge": four people swing in, lock
//! together, then pull in tight. It plays once and ends on the static mark.
//!
//! The geometry, timing and easing are the brand animation's own keyframes
//! (1080 × 1080 canvas, 60 frames a second, 90 frames), drawn natively: one
//! person is an arm and a head, and the mark is that person four times, a
//! quarter turn apart. Nothing here may be re-timed or redrawn without the
//! brand file changing first.

use std::time::Duration;

use isonomic_app::ui::{point, px, rgb, Bounds, Hsla, PathBuilder, Pixels, Point, Window};

/// Frames a second, and the frame the intro ends on.
const FPS: f32 = 60.;
pub const LAST_FRAME: f32 = 90.;

/// The mark spans this many units either side of its centre: 116 across.
const HALF_EXTENT: f32 = 58.;

/// A person's arm: each vertex with its incoming and outgoing tangents.
const ARM: [([f32; 2], [f32; 2], [f32; 2]); 6] = [
    ([-13.643, -58.], [0., 0.], [0., 10.812]),
    ([-1.85, -29.077], [-7.56, -7.729], [7.56, 7.729]),
    ([26.804, -16.647], [-10.809, -0.24], [-4.858, 3.416]),
    ([14.182, -4.242], [3.5, -4.798], [-11.973, -3.015]),
    ([-16.005, -23.921], [7.59, 9.738], [-7.59, -9.738]),
    ([-27.716, -58.], [0., 12.347], [0., 0.]),
];
/// A person's head: a circle.
const HEAD_CENTER: [f32; 2] = [10.239, -40.514];
const HEAD_RADIUS: f32 = 12.3675;
/// Control distance for a quarter circle drawn as one cubic curve.
const KAPPA: f32 = 0.552_284_8;

/// Each person starts this many frames after the one before.
const STAGGER: f32 = 4.8;
/// A person fades in over this many frames, evenly.
const FADE: f32 = 4.104;
/// A person swings and drops into place over this many frames.
const SWING: f32 = 54.;
/// Where the swing starts, relative to the person's place: degrees, and
/// units above the centre.
const SWING_FROM: f32 = -80.;
const DROP_FROM: f32 = -84.;
/// The swing's easing, as a cubic curve's two control points.
const SWING_EASE: [f32; 4] = [0.16, 1., 0.3, 1.];

/// The whole mark's scale once the people have locked: the frame, the
/// scale relative to the resting mark, and the easing to the next key.
const PULL: [(f32, f32, [f32; 4]); 4] = [
    (54., 1., [0.3, 0., 0.3, 1.]),
    (67.68, 448.393 / 482.143, [0.3, 0., 0.2, 1.]),
    (81., 489.375 / 482.143, [0.42, 0., 0.58, 1.]),
    (90., 1., [0., 0., 1., 1.]),
];

/// How long the intro runs.
pub fn duration() -> Duration {
    Duration::from_secs_f32(LAST_FRAME / FPS)
}

/// The frame shown `elapsed` after the start, held on the last one.
pub fn frame_at(elapsed: Duration) -> f32 {
    (elapsed.as_secs_f32() * FPS).min(LAST_FRAME)
}

/// The mark's colour on a light background.
pub fn mark_color() -> Hsla {
    rgb(0x111827).into()
}

/// One person at one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub opacity: f32,
    /// Degrees, clockwise.
    pub rotation: f32,
    /// Units along the person's own axis; negative is away from the centre.
    pub lift: f32,
}

/// A cubic easing curve through (0, 0) and (1, 1): the eased value at `x`.
fn ease([x1, y1, x2, y2]: [f32; 4], x: f32) -> f32 {
    let x = x.clamp(0., 1.);
    let at = |a: f32, b: f32, t: f32| {
        let u = 1. - t;
        3. * u * u * t * a + 3. * u * t * t * b + t * t * t
    };
    // The curve's x is monotonic, so bisect for the parameter.
    let (mut low, mut high) = (0., 1.);
    for _ in 0..24 {
        let mid = (low + high) / 2.;
        if at(x1, x2, mid) < x {
            low = mid;
        } else {
            high = mid;
        }
    }
    at(y1, y2, (low + high) / 2.)
}

/// Person `index` (0 to 3) at `frame`.
pub fn pose(index: usize, frame: f32) -> Pose {
    let start = STAGGER * index as f32;
    let place = 90. * index as f32;
    let swung = ease(SWING_EASE, (frame - start) / SWING);
    Pose {
        opacity: ((frame - start) / FADE).clamp(0., 1.),
        rotation: place + SWING_FROM * (1. - swung),
        lift: DROP_FROM * (1. - swung),
    }
}

/// The whole mark's scale at `frame`, relative to the resting mark.
pub fn pull(frame: f32) -> f32 {
    for pair in PULL.windows(2) {
        let ((from, a, easing), (to, b, _)) = (pair[0], pair[1]);
        if frame < to {
            let eased = ease(easing, (frame - from) / (to - from));
            return a + (b - a) * eased;
        }
    }
    1.
}

/// Draw the mark at `frame`, centred in `bounds` and sized so the resting
/// mark fills its width. While people swing in they reach outside it.
pub fn paint(bounds: Bounds<Pixels>, frame: f32, color: Hsla, window: &mut Window) {
    let center = bounds.center();
    let (cx, cy) = (f32::from(center.x), f32::from(center.y));
    let unit = f32::from(bounds.size.width) / (2. * HALF_EXTENT) * pull(frame);
    for index in 0..4 {
        let pose = pose(index, frame);
        if pose.opacity <= 0. {
            continue;
        }
        let (sin, cos) = pose.rotation.to_radians().sin_cos();
        let place = |[x, y]: [f32; 2]| -> Point<Pixels> {
            let y = y + pose.lift;
            point(
                px(cx + (x * cos - y * sin) * unit),
                px(cy + (x * sin + y * cos) * unit),
            )
        };
        let add = |a: [f32; 2], b: [f32; 2]| [a[0] + b[0], a[1] + b[1]];
        let color = color.opacity(pose.opacity);

        let mut arm = PathBuilder::fill();
        arm.move_to(place(ARM[0].0));
        for at in 0..ARM.len() {
            let (from, _, out) = ARM[at];
            let (to, into, _) = ARM[(at + 1) % ARM.len()];
            arm.cubic_bezier_to(place(to), place(add(from, out)), place(add(to, into)));
        }
        arm.close();
        if let Ok(arm) = arm.build() {
            window.paint_path(arm, color);
        }

        let [hx, hy] = HEAD_CENTER;
        let (r, k) = (HEAD_RADIUS, HEAD_RADIUS * KAPPA);
        let mut head = PathBuilder::fill();
        head.move_to(place([hx, hy - r]));
        head.cubic_bezier_to(
            place([hx + r, hy]),
            place([hx + k, hy - r]),
            place([hx + r, hy - k]),
        );
        head.cubic_bezier_to(
            place([hx, hy + r]),
            place([hx + r, hy + k]),
            place([hx + k, hy + r]),
        );
        head.cubic_bezier_to(
            place([hx - r, hy]),
            place([hx - k, hy + r]),
            place([hx - r, hy + k]),
        );
        head.cubic_bezier_to(
            place([hx, hy - r]),
            place([hx - r, hy - k]),
            place([hx - k, hy - r]),
        );
        head.close();
        if let Ok(head) = head.build() {
            window.paint_path(head, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn it_runs_a_second_and_a_half_and_holds_the_last_frame() {
        assert_eq!(duration(), Duration::from_millis(1500));
        assert!(close(frame_at(Duration::from_millis(500)), 30.));
        assert_eq!(frame_at(Duration::from_secs(9)), LAST_FRAME);
    }

    #[test]
    fn each_person_starts_unseen_up_and_turned_back() {
        for index in 0..4 {
            let start = STAGGER * index as f32;
            let first = pose(index, start);
            assert_eq!(first.opacity, 0.);
            assert!(close(first.rotation, 90. * index as f32 - 80.));
            assert!(close(first.lift, -84.));
            // Before its turn, it is not drawn at all.
            assert_eq!(pose(index, start - 1.).opacity.max(0.), 0.);
            assert!(close(pose(index, start + FADE).opacity, 1.));
        }
    }

    #[test]
    fn it_ends_on_the_static_mark() {
        for index in 0..4 {
            let last = pose(index, LAST_FRAME);
            assert_eq!(last.opacity, 1.);
            assert!(close(last.rotation, 90. * index as f32));
            assert!(close(last.lift, 0.));
        }
        assert!(close(pull(LAST_FRAME), 1.));
    }

    #[test]
    fn the_last_person_lands_before_the_end() {
        let landed = STAGGER * 3. + SWING;
        assert!(landed < LAST_FRAME);
        assert!(close(pose(3, landed).lift, 0.));
    }

    #[test]
    fn the_mark_pulls_in_then_settles() {
        assert!(close(pull(0.), 1.));
        assert!(close(pull(54.), 1.));
        assert!(close(pull(67.68), 448.393 / 482.143));
        assert!(close(pull(81.), 489.375 / 482.143));
        assert!(pull(60.) < 1. && pull(75.) > pull(67.68));
    }

    #[test]
    fn the_swing_starts_fast_and_ends_slow() {
        assert!(ease(SWING_EASE, 0.25) > 0.7);
        assert!(close(ease(SWING_EASE, 0.), 0.));
        assert!(close(ease(SWING_EASE, 1.), 1.));
        assert!(close(ease([0., 0., 1., 1.], 0.4), 0.4));
    }
}

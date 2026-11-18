// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Easing shared by the window's animations.

/// Fast out of the gate, a long soft landing: the brand intro's own swing.
pub const SWING: [f32; 4] = [0.16, 1., 0.3, 1.];

/// Out quickly, then a slower, even landing: for something large crossing
/// the window, whose whole path should be seen.
pub const GLIDE: [f32; 4] = [0.33, 1., 0.68, 1.];

/// Slow to start, then gone: for something leaving the window.
pub const DEPART: [f32; 4] = [0.5, 0., 0.9, 0.5];

/// A cubic easing curve through (0, 0) and (1, 1), given as its two
/// control points: the eased value at `x`.
pub fn ease([x1, y1, x2, y2]: [f32; 4], x: f32) -> f32 {
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

/// How far through a step that starts at `start` and lasts `length` the
/// time `now` is, from 0 to 1.
pub fn progress(now: f32, start: f32, length: f32) -> f32 {
    ((now - start) / length).clamp(0., 1.)
}

/// `from` to `to`, `t` of the way.
pub fn between(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn the_swing_starts_fast_and_ends_slow() {
        assert!(ease(SWING, 0.25) > 0.7);
        assert!(close(ease(SWING, 0.), 0.));
        assert!(close(ease(SWING, 1.), 1.));
        assert!(close(ease([0., 0., 1., 1.], 0.4), 0.4));
    }

    #[test]
    fn progress_is_held_at_both_ends() {
        assert_eq!(progress(1., 2., 4.), 0.);
        assert!(close(progress(3., 2., 4.), 0.25));
        assert_eq!(progress(9., 2., 4.), 1.);
        assert!(close(between(10., 20., 0.5), 15.));
    }
}

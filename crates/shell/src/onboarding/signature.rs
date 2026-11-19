// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The signature the pen writes: "hello" in one joined line, as points in
//! a box one wide and one tall, from its bottom left. Our own hand.

use std::sync::OnceLock;

/// The pen's turning points, in the order it passes them.
const TURNS: [(f32, f32); 42] = [
    (0.00, 0.12),
    (0.07, 0.42),
    (0.125, 0.85),
    (0.105, 1.0),
    (0.075, 0.80),
    (0.065, 0.40),
    (0.06, 0.0),
    (0.085, 0.30),
    (0.14, 0.46),
    (0.175, 0.34),
    (0.18, 0.10),
    (0.215, 0.0),
    (0.275, 0.14),
    (0.325, 0.38),
    (0.29, 0.47),
    (0.255, 0.28),
    (0.29, 0.04),
    (0.36, 0.04),
    (0.43, 0.40),
    (0.475, 0.88),
    (0.45, 1.0),
    (0.415, 0.80),
    (0.41, 0.30),
    (0.44, 0.02),
    (0.50, 0.04),
    (0.565, 0.40),
    (0.61, 0.88),
    (0.585, 1.0),
    (0.55, 0.80),
    (0.545, 0.30),
    (0.575, 0.02),
    (0.64, 0.05),
    (0.70, 0.22),
    (0.755, 0.44),
    (0.815, 0.40),
    (0.835, 0.20),
    (0.79, 0.02),
    (0.725, 0.08),
    (0.715, 0.30),
    (0.775, 0.45),
    (0.88, 0.40),
    (1.0, 0.46),
];
/// Points drawn between each two turning points.
const STEPS: usize = 8;

/// A smooth curve through four points, `t` of the way from the second to
/// the third.
fn through(p: [(f32, f32); 4], t: f32) -> (f32, f32) {
    let one = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2. * b
            + (c - a) * t
            + (2. * a - 5. * b + 4. * c - d) * t * t
            + (3. * b - a - 3. * c + d) * t * t * t)
    };
    (
        one(p[0].0, p[1].0, p[2].0, p[3].0),
        one(p[0].1, p[1].1, p[2].1, p[3].1),
    )
}

/// The whole line, as points close enough together to be joined straight,
/// each with the share of the line's length written when the pen is there.
pub fn line() -> &'static [((f32, f32), f32)] {
    static LINE: OnceLock<Vec<((f32, f32), f32)>> = OnceLock::new();
    LINE.get_or_init(|| {
        let last = TURNS.len() - 1;
        let mut points = Vec::with_capacity(last * STEPS + 1);
        for at in 0..last {
            let four = [
                TURNS[at.saturating_sub(1)],
                TURNS[at],
                TURNS[at + 1],
                TURNS[(at + 2).min(last)],
            ];
            for step in 0..STEPS {
                points.push(through(four, step as f32 / STEPS as f32));
            }
        }
        points.push(TURNS[last]);
        let mut length = 0.;
        let mut lengths = vec![0.];
        for pair in points.windows(2) {
            length += (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1);
            lengths.push(length);
        }
        points
            .into_iter()
            .zip(lengths)
            .map(|(point, written)| (point, written / length))
            .collect()
    })
}

/// The points written when `share` of the line is, ending where the pen
/// is.
pub fn written(share: f32) -> Vec<(f32, f32)> {
    let line = line();
    let share = share.clamp(0., 1.);
    let whole = line.partition_point(|(_, at)| *at <= share);
    let mut points: Vec<(f32, f32)> = line[..whole].iter().map(|(point, _)| *point).collect();
    if let (Some(&(from, a)), Some(&(to, b))) = (line.get(whole.wrapping_sub(1)), line.get(whole)) {
        let part = (share - a) / (b - a).max(f32::EPSILON);
        points.push((
            from.0 + (to.0 - from.0) * part,
            from.1 + (to.1 - from.1) * part,
        ));
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_runs_left_to_right_inside_its_box() {
        let line = line();
        assert_eq!(line.first().unwrap().1, 0.);
        assert!((line.last().unwrap().1 - 1.).abs() < 1e-5);
        assert!(line.first().unwrap().0 .0 < 0.01 && line.last().unwrap().0 .0 > 0.99);
        assert!(line
            .iter()
            .all(|((x, y), _)| (-0.05..=1.05).contains(x) && (-0.05..=1.05).contains(y)));
    }

    #[test]
    fn more_is_written_as_the_pen_goes() {
        assert_eq!(written(0.).len(), 2);
        let half = written(0.5);
        assert!(half.len() > 100 && half.len() < written(1.).len());
        assert_eq!(*written(1.).last().unwrap(), TURNS[TURNS.len() - 1]);
    }
}

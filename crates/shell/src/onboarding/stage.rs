// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! One drawn frame of a 3D model, as an image the window can show. Slow:
//! everything here is called off the foreground thread.

use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;
use std::sync::{Arc, LazyLock, Mutex};

use device3d::{Camera, Model, Shot, Transform};
use image::{Frame, RgbaImage};
use ui::RenderImage;

/// A drawn frame and the time on its screen's clock it was drawn for.
/// It is placed by that time, so where it is and how far it has turned
/// always agree.
pub struct Drawn {
    pub image: Arc<RenderImage>,
    pub at: f32,
}

/// The frames drawn so far, by what was drawn. A move drawn once, ahead
/// of time or as it played, plays again and plays backward without being
/// drawn again. The oldest go when there are too many.
#[derive(Default)]
struct Kept {
    frames: HashMap<u64, Arc<RenderImage>>,
    order: VecDeque<(u64, usize)>,
    bytes: usize,
}

static KEPT: LazyLock<Mutex<Kept>> = LazyLock::new(Mutex::default);
/// How many bytes of frames are kept.
const KEEP: usize = 384 << 20;

/// Forget every kept frame: onboarding is over.
pub fn forget() {
    if let Ok(mut kept) = KEPT.lock() {
        *kept = Kept::default();
    }
}

/// The screen's ticks, sixty a second. Frames are drawn for times on a
/// tick, so that the same moment asked for twice is the same frame.
pub const TICK: f32 = 1. / 60.;

/// The tick nearest `t`.
pub fn snap(t: f32) -> f32 {
    (t / TICK).round() * TICK
}

/// Every tick from `from` to `to`.
pub fn ticks(from: f32, to: f32) -> impl Iterator<Item = f32> {
    ((from / TICK).floor() as i32..=(to / TICK).ceil() as i32).map(|tick| tick as f32 * TICK)
}

thread_local! {
    /// Is this thread drawing ahead?
    static AHEAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `draw`, which draws frames ahead of their time. It takes a third
/// of the processor, so what is on screen meanwhile keeps its pace.
pub fn ahead(draw: impl FnOnce()) {
    AHEAD.set(true);
    draw();
    AHEAD.set(false);
}

/// Samples along each side of a pixel.
const SAMPLES: usize = 2;

/// A display's light and the linear colours of its picture's corners.
pub struct Display {
    pub light: f32,
    pub picture: [[f32; 3]; 4],
}

impl Display {
    pub const NONE: Display = Display {
        light: 0.,
        picture: [[0.; 3]; 4],
    };
}

/// A model, the camera it is seen by, and any of its pieces moved on
/// their own.
pub type Subject<'a> = (&'a Model, &'a Camera, &'a [(Range<u32>, Transform)]);

/// Draw `model` posed by `pose` on a canvas of `canvas` points, in which
/// the camera's frame is `frame` points wide, for a screen with `scale`
/// pixels to a point, in full detail or, to keep a large model's frames
/// coming while it moves, in half.
pub fn frame(
    (model, camera, parts): Subject<'_>,
    pose: Transform,
    full: bool,
    canvas: (f32, f32),
    frame: f32,
    scale: f32,
    display: Display,
) -> Arc<RenderImage> {
    // What is drawn, as a number: the same asked for again is the frame
    // already drawn.
    let mut what = DefaultHasher::new();
    (std::ptr::from_ref(model) as usize, model.vertices.len()).hash(&mut what);
    format!("{camera:?}").hash(&mut what);
    let turns = std::iter::once(&pose).chain(parts.iter().map(|(_, turn)| turn));
    for turn in turns {
        for axis in turn.rows.iter().chain([&turn.shift]) {
            [axis.x.to_bits(), axis.y.to_bits(), axis.z.to_bits()].hash(&mut what);
        }
    }
    for (piece, _) in parts {
        (piece.start, piece.end).hash(&mut what);
    }
    let numbers = [canvas.0, canvas.1, frame, scale, display.light];
    for number in numbers.iter().chain(display.picture.iter().flatten()) {
        number.to_bits().hash(&mut what);
    }
    full.hash(&mut what);
    let what = what.finish();
    if let Some(drawn) = KEPT
        .lock()
        .ok()
        .and_then(|kept| kept.frames.get(&what).cloned())
    {
        return drawn;
    }
    let scale = if full { scale } else { scale * 0.5 };
    let shot = Shot {
        model,
        camera,
        pose,
        parts,
        width: (canvas.0 * scale).round().max(1.) as usize,
        height: (canvas.1 * scale).round().max(1.) as usize,
        frame_width: frame * scale,
        samples: SAMPLES,
        light: display.light,
        picture: display.picture,
    };
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let threads = if AHEAD.get() {
        (threads / 3).max(2)
    } else {
        threads
    };
    let drawn = device3d::render(&shot, threads);
    let size = drawn.bgra.len();
    let pixels = RgbaImage::from_raw(drawn.width as u32, drawn.height as u32, drawn.bgra)
        .expect("a frame holds four bytes a pixel");
    let drawn = Arc::new(RenderImage::new(vec![Frame::new(pixels)]));
    if let Ok(mut kept) = KEPT.lock() {
        if kept.frames.insert(what, drawn.clone()).is_none() {
            kept.order.push_back((what, size));
            kept.bytes += size;
        }
        while kept.bytes > KEEP {
            let Some((old, size)) = kept.order.pop_front() else {
                break;
            };
            kept.frames.remove(&old);
            kept.bytes -= size;
        }
    }
    drawn
}

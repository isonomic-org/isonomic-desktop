// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recovery kit's titanium holder and its padlock: their 3D models
//! from the devices repository, drawn from above as they lie.
//!
//! The holder is two plates on a hinge, each with twelve words of four
//! letter tiles. Its top plate folds over onto the bottom one, which
//! brings the two lock loops together and shows the mark cut into its
//! back. Each tile can lift out of its window and turn about its row's
//! line. The padlock's body slides along its shackle.

use std::ops::Range;
use std::sync::Arc;

use device3d::{vec3, Camera, Framing, Model, ModelError, Transform, Vec3};
use ui::onboarding::kit::{self as design, at};
use ui::onboarding::terms::CLIPBOARD_ELEVATION as FROM_ABOVE;
use ui::{motion, RenderImage};

use super::stage::{self, Display};

static HOLDER: &[u8] = include_bytes!("../../../../assets/onboarding/recovery-kit-holder.glb");
static PADLOCK: &[u8] = include_bytes!("../../../../assets/onboarding/recovery-kit-padlock.glb");

/// The holder's model, in its own metres: a point on the hinge's line,
/// which runs across; how far a tile lifts to turn; how far up the page
/// the closed holder lies, so the padlock has room under it; and how far
/// in from a loop's tip its hole is.
const HINGE: Vec3 = vec3(0., 0.0028, 0.00038);
const ACROSS: Vec3 = vec3(1., 0., 0.);
const TILE_LIFT: f32 = 0.0016;
const CLOSED_RISE: f32 = 0.045;
const HOLE: f32 = 0.0056;
/// The padlock's: how far its body hangs off its shackle when open, and
/// half the thickness of the shackle's bar.
const LOCK_OPEN: f32 = 0.005;
const HALF_BAR: f32 = 0.0024;
/// The first and last pieces of the top plate, in the file's order, and
/// what its tiles' names start with.
const TOP: (&str, &str) = ("face_0", "tile_0_3_11");
const MARK: &str = "G_body__mark";
const MARK_SHADE: f32 = 0.45;
const TILE: &str = "tile_";
const TOP_TILE: &str = "tile_0_";

/// How the holder lies, in degrees: the turn it has left to make as it
/// comes or goes (like a coin, about the line up the page); how far it has
/// turned its back (about the hinge's line); how far its top plate is
/// folded over (180 is closed); how far it has risen to where it lies
/// closed (0 to 1); and how long its tiles have been turning, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub turn: f32,
    pub away: f32,
    pub fold: f32,
    pub risen: f32,
    pub spun: f32,
}

impl Pose {
    /// Open, its words up, every tile in place.
    pub const OPEN: Pose = Pose {
        turn: 0.,
        away: 0.,
        fold: 0.,
        risen: 0.,
        spun: f32::MAX,
    };
    /// Open, its back to the person.
    pub const AWAY: Pose = Pose {
        away: 180.,
        ..Pose::OPEN
    };
    /// Closed, its mark up.
    pub const CLOSED: Pose = Pose {
        fold: 180.,
        risen: 1.,
        ..Pose::OPEN
    };
}

/// How long the tiles take to turn, first to last.
pub fn spinning(tiles: usize) -> f32 {
    at::SPIN + at::SPIN_STAGGER * tiles.saturating_sub(1) as f32
}

/// A tile `s` seconds after it set off: how far round it is, in radians,
/// and how far it is lifted, from 0 to 1.
fn tile(s: f32) -> (f32, f32) {
    let share = motion::progress(s, 0., at::SPIN);
    if share <= 0. || share >= 1. {
        return (0., 0.);
    }
    (
        std::f32::consts::TAU * motion::ease(motion::CARRY, share),
        (std::f32::consts::PI * share).sin(),
    )
}

fn moved(by: Vec3) -> Transform {
    Transform {
        shift: by,
        ..Transform::IDENTITY
    }
}

fn middle(model: &Model, vertices: &Range<u32>) -> (Vec3, Vec3) {
    let mut least = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut most = vec3(f32::MIN, f32::MIN, f32::MIN);
    for index in vertices.clone() {
        let at = model.vertices[index as usize].position;
        (least, most) = (least.min(at), most.max(at));
    }
    (least, most)
}

pub struct Kit {
    holder: (Model, Camera),
    /// Everything on the top plate.
    top: Range<u32>,
    /// Each tile in reading order: its vertices, its middle, and whether
    /// it is on the top plate.
    tiles: Vec<(Range<u32>, Vec3, bool)>,
    /// Where the hole of the bottom plate's loop is.
    hole: Vec3,
    lock: (Model, Camera),
    body: Range<u32>,
    /// The middle of the top of the shackle's bar.
    bar: Vec3,
}

impl Kit {
    /// Read both models. Slow enough to keep off the foreground thread.
    pub fn load() -> Result<Kit, ModelError> {
        let mut holder = Model::read(HOLDER)?;
        // The mark is cut into the plate, and the file gives it the plate's
        // own colour: its floor is in shade.
        for material in &mut holder.materials {
            if material.name == MARK {
                material.base = material.base.map(|channel| channel * MARK_SHADE);
            }
        }
        let lost = || ModelError::Malformed("a piece of the recovery kit");
        let top =
            holder.part(TOP.0).ok_or_else(lost)?.start..holder.part(TOP.1).ok_or_else(lost)?.end;
        let tiles = holder
            .parts
            .iter()
            .filter(|part| part.name.starts_with(TILE))
            .map(|part| {
                let (least, most) = middle(&holder, &part.vertices);
                (
                    part.vertices.clone(),
                    (least + most) * 0.5,
                    part.name.starts_with(TOP_TILE),
                )
            })
            .collect();
        let (least, most) = middle(&holder, &holder.part("loop.001").ok_or_else(lost)?);
        let hole = vec3((least.x + most.x) / 2., most.y, most.z - HOLE);
        // Seen from above, as it lies: open, it is `OPEN_TALL` tall.
        let (wide, tall) = design::CANVAS;
        let above = Camera::front(
            &holder,
            FROM_ABOVE,
            Framing {
                fill: design::OPEN_TALL / tall,
                lift: 0.,
                aspect: tall / wide,
            },
        );
        let lock = Model::read(PADLOCK)?;
        let body = lock.part("lock_body").ok_or_else(lost)?;
        let (least, most) = middle(&lock, &lock.part("shackle").ok_or_else(lost)?);
        let bar = vec3((least.x + most.x) / 2., most.y, least.z + HALF_BAR);
        // The padlock at the holder's own scale.
        let (lock_wide, lock_tall) = design::LOCK_CANVAS;
        let share = (lock.max.z - lock.min.z) / (holder.max.z - holder.min.z);
        let beside = Camera::front(
            &lock,
            FROM_ABOVE,
            Framing {
                fill: design::OPEN_TALL * share / lock_tall,
                lift: 0.,
                aspect: lock_tall / lock_wide,
            },
        );
        Ok(Kit {
            holder: (holder, above),
            top,
            tiles,
            hole,
            lock: (lock, beside),
            body,
            bar,
        })
    }

    /// How many letter tiles the holder has.
    pub fn tiles(&self) -> usize {
        self.tiles.len()
    }

    /// The holder as `pose` has it.
    pub fn holder(&self, pose: Pose, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.holder;
        let fold = Transform::turn_about(ACROSS, pose.fold.to_radians(), HINGE);
        let mut parts = Vec::new();
        for (index, (vertices, middle, top)) in self.tiles.iter().enumerate() {
            let (round, lift) = tile(pose.spun - at::SPIN_STAGGER * index as f32);
            if round == 0. {
                continue;
            }
            let turned = moved(vec3(0., TILE_LIFT * lift, 0.))
                .after(&Transform::turn_about(ACROSS, round, *middle));
            let turned = if *top { fold.after(&turned) } else { turned };
            parts.push((vertices.clone(), turned));
        }
        if pose.fold != 0. {
            parts.push((self.top.clone(), fold));
        }
        let centre = model.centre();
        let away = Transform::turn_about(ACROSS, pose.away.to_radians(), centre);
        let risen = moved(vec3(0., 0., -CLOSED_RISE * pose.risen));
        let turn = Transform::turn_about(camera.up, pose.turn.to_radians(), centre);
        stage::frame(
            (model, camera, &parts),
            turn.after(&risen.after(&away)),
            true,
            design::CANVAS,
            design::CANVAS.0,
            scale,
            Display::NONE,
        )
    }

    /// The padlock, its body `clipped` of the way up its shackle: 0 is
    /// open, 1 shut.
    pub fn lock(&self, clipped: f32, scale: f32) -> Arc<RenderImage> {
        let (model, camera) = &self.lock;
        let hangs = moved(vec3(0., 0., LOCK_OPEN * (1. - clipped)));
        stage::frame(
            (model, camera, &[(self.body.clone(), hangs)]),
            Transform::IDENTITY,
            true,
            design::LOCK_CANVAS,
            design::LOCK_CANVAS.0,
            scale,
            Display::NONE,
        )
    }

    /// Where the middle of the padlock's canvas goes, in points right of
    /// and below the middle of the holder's, for the top of its shackle to
    /// lie over the hole of the closed holder's loops.
    pub fn lock_place(&self) -> (f32, f32) {
        let (hole_u, hole_v, _) = self.holder.1.project(self.hole - vec3(0., 0., CLOSED_RISE));
        let (bar_u, bar_v, _) = self.lock.1.project(self.bar);
        (
            hole_u * design::CANVAS.0 - bar_u * design::LOCK_CANVAS.0,
            -(hole_v * design::CANVAS.0 - bar_v * design::LOCK_CANVAS.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kit_loads_with_its_ninety_six_tiles() {
        let kit = Kit::load().unwrap();
        assert_eq!(kit.tiles(), 96);
        assert_eq!(kit.tiles.iter().filter(|tile| tile.2).count(), 48);
        // Every piece between the top plate's first and last is its own.
        let (model, _) = &kit.holder;
        assert!(model
            .parts
            .iter()
            .filter(|part| kit.top.contains(&part.vertices.start))
            .all(|part| part.name != "plate_1" && !part.name.starts_with("tile_1_")));
        assert!(kit.top.contains(&model.part("plate_0").unwrap().start));
        assert!(kit.top.contains(&model.part("mark").unwrap().start));
        assert_eq!(kit.holder(Pose::OPEN, 1.).size(0).width.0, 220);
    }

    #[test]
    fn the_padlock_hangs_under_the_closed_holder() {
        let kit = Kit::load().unwrap();
        let (across, down) = kit.lock_place();
        assert!(
            across.abs() < 6. && down > 20. && down < 80.,
            "{across} {down}"
        );
    }

    #[test]
    fn a_tile_lifts_turns_once_and_drops_back() {
        assert_eq!(tile(-0.1), (0., 0.));
        assert_eq!(tile(at::SPIN), (0., 0.));
        let (round, lift) = tile(at::SPIN / 2.);
        assert!(round > 1. && round < 6. && lift > 0.9);
    }
}

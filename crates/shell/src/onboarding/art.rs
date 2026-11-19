// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The art in the sidebar for each of onboarding's later steps, as
//! sprites: a picture and where it rests. Where the devices repository has
//! a 3D model the sprite is that model drawn once: the pills of "Add
//! integrations", the wallet and the ID card from above, the drives of
//! "Local AI models" tilted as the design has them. Where it has none
//! yet, it is the design's own flat picture.
//!
//! This is a first pass: a sprite turns over by being drawn narrower, not
//! by its model being drawn again.

use std::sync::Arc;

use device3d::{Camera, Framing, Model, Transform};
use ui::onboarding::steps as design;
use ui::onboarding::terms::CLIPBOARD_ELEVATION as FROM_ABOVE;
use ui::{Image, ImageFormat, ImageSource, RenderImage};

use super::stage::{self, Display};
use super::steps;

/// The scale the models are drawn at: two pixels to a point is the most
/// any of this machine's screens asks.
const SCALE: f32 = 2.;

#[derive(Clone)]
pub enum Picture {
    Drawn(Arc<RenderImage>),
    Flat(Arc<Image>),
}

impl From<&Picture> for ImageSource {
    fn from(picture: &Picture) -> ImageSource {
        match picture {
            Picture::Drawn(image) => image.clone().into(),
            Picture::Flat(image) => image.clone().into(),
        }
    }
}

/// A box in the window's points: its corner, then its size.
pub type Place = (f32, f32, f32, f32);

/// A picture and the box it rests in.
pub struct Sprite {
    pub picture: Picture,
    pub place: Place,
    /// Is this the ID card, which slips into the wallet?
    pub slips: bool,
}

/// Each step's sprites, back to front.
pub struct Art {
    pub steps: [Vec<Sprite>; 6],
}

macro_rules! asset {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/onboarding/", $name)).as_slice()
    };
}

fn flat(png: &'static [u8], place: Place) -> Vec<Sprite> {
    vec![Sprite {
        picture: Picture::Flat(Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec()))),
        place,
        slips: false,
    }]
}

/// A row of models, each drawn once to fill its box by the camera `seen`
/// stands before it.
fn row(
    models: &[(&'static [u8], Place)],
    fill: f32,
    seen: impl Fn(&Model, Framing) -> Camera,
) -> Vec<Sprite> {
    models
        .iter()
        .filter_map(|(bytes, place)| {
            let model = Model::read(bytes).ok()?;
            let (_, _, wide, tall) = *place;
            let camera = seen(
                &model,
                Framing {
                    fill,
                    lift: 0.,
                    aspect: tall / wide,
                },
            );
            let image = stage::frame(
                (&model, &camera, &[]),
                Transform::IDENTITY,
                true,
                (wide, tall),
                wide,
                SCALE,
                Display::NONE,
            );
            Some(Sprite {
                picture: Picture::Drawn(image),
                place: *place,
                slips: false,
            })
        })
        .collect()
}

/// One model drawn from above to fill `place`, keeping only the pieces
/// `kept` says to: the whole model still frames the camera, so the pieces
/// of one model drawn apart still fit together.
fn pieces(bytes: &[u8], place: Place, kept: impl Fn(&str) -> bool) -> Option<Picture> {
    let mut model = Model::read(bytes).ok()?;
    let (_, _, wide, tall) = place;
    let camera = Camera::front(
        &model,
        FROM_ABOVE,
        Framing {
            fill: 1.,
            lift: 0.,
            aspect: tall / wide,
        },
    );
    let dropped: Vec<_> = model
        .parts
        .iter()
        .filter(|part| !kept(&part.name))
        .map(|part| part.vertices.clone())
        .collect();
    model.triangles.retain(|triangle| {
        !dropped
            .iter()
            .any(|part| part.contains(&triangle.vertices[0]))
    });
    Some(Picture::Drawn(stage::frame(
        (&model, &camera, &[]),
        Transform::IDENTITY,
        true,
        (wide, tall),
        wide,
        SCALE,
        Display::NONE,
    )))
}

/// The card wallet and the ID card, as three sprites so that the card can
/// slip between the wallet's back and its pocket.
fn wallet_and_card() -> Vec<Sprite> {
    let wallet = asset!("wallet-cream.glb");
    let sprite = |picture: Option<Picture>, place: Place, slips: bool| {
        picture.map(|picture| Sprite {
            picture,
            place,
            slips,
        })
    };
    [
        sprite(
            pieces(wallet, design::WALLET, |part| part == "back"),
            design::WALLET,
            false,
        ),
        sprite(
            pieces(asset!("id-card-black.glb"), design::CARD, |_| true),
            design::CARD,
            true,
        ),
        sprite(
            pieces(wallet, design::WALLET, |part| part != "back"),
            design::WALLET,
            false,
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

impl Art {
    /// Read and draw everything. Slow: keep it off the foreground thread.
    pub fn load() -> Art {
        let mut art = Art {
            steps: Default::default(),
        };
        // The design's flat pictures, each on a clear ground, at three
        // pixels to a point.
        art.steps[steps::CREATED] = wallet_and_card();
        art.steps[steps::KIT] = flat(
            asset!("recovery-kit-holder.png"),
            (123.2, 454.5, 150.7, 184.),
        );
        art.steps[steps::CONFIRM] = flat(asset!("recovery-kit-seal.png"), (125., 450., 150., 150.));
        art.steps[steps::FINISH] = flat(asset!("menu-bar.png"), (60., 460., 304., 56.));
        // The drives, the middle one nearest; and the pills likewise.
        let [gemma, hugging_face, mistral, microsoft, qwen] = design::DRIVES;
        art.steps[steps::MODELS] = row(
            &[
                (asset!("drive-gemma.glb"), gemma),
                (asset!("drive-hugging-face.glb"), hugging_face),
                (asset!("drive-mistral.glb"), mistral),
                (asset!("drive-microsoft.glb"), microsoft),
                (asset!("drive-qwen.glb"), qwen),
            ],
            1.,
            |model, framing| Camera::axonometric(model, design::DRIVE_ELEVATION, framing),
        );
        let from_above = |model: &Model, framing| Camera::front(model, FROM_ABOVE, framing);
        art.steps[steps::LINKS] = row(
            &[
                (asset!("pill-dropbox.glb"), (36., 502., 40., 40.)),
                (asset!("pill-github.glb"), (324., 502., 40., 40.)),
                (asset!("pill-asana.glb"), (90., 496., 52., 52.)),
                (asset!("pill-figma.glb"), (258., 496., 52., 52.)),
                (asset!("pill-linear.glb"), (156., 478., 88., 88.)),
            ],
            0.98,
            from_above,
        );
        art
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_step_has_its_art() {
        let art = Art::load();
        assert!(art.steps.iter().all(|sprites| !sprites.is_empty()));
        assert_eq!(art.steps[steps::CREATED].len(), 3);
        assert!(art.steps[steps::CREATED][1].slips);
        assert_eq!(art.steps[steps::MODELS].len(), 5);
        assert_eq!(art.steps[steps::LINKS].len(), 5);
    }
}

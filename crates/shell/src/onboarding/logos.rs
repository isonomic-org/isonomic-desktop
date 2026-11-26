// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The marks beside the rows of "Turn on local AI models" and
//! "Integrations and connected apps": the design's own pictures, each in
//! its 40-point box, exported at three pixels to a point.

use std::sync::{Arc, OnceLock};

use ui::{Image, ImageFormat};

use super::steps;

macro_rules! logo {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../../../assets/onboarding/logos/",
            $name,
            ".png"
        ))
        .as_slice()
    };
}

/// The makers' marks the design has, each with how a model's name begins
/// when it is that maker's.
const MAKERS: [(&str, &[u8]); 6] = [
    ("qwen", logo!("qwen")),
    ("mistral", logo!("mistral")),
    ("gemma", logo!("gemma")),
    ("phi", logo!("microsoft")),
    ("smol", logo!("hugging-face")),
    ("gpt", logo!("openai")),
];
const LINKS: [&[u8]; steps::LINK_LIST.len()] = [
    logo!("linear"),
    logo!("figma"),
    logo!("github"),
    logo!("zapier"),
    logo!("notion"),
    logo!("loom"),
    logo!("mailchimp"),
    logo!("dropbox"),
];

fn read(png: &'static [u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec()))
}

/// The mark of the maker of the model called `name`, where the design has
/// it. Each is read once and kept, so a row shows the same picture every
/// time it is drawn.
pub fn model(name: &str) -> Option<Arc<Image>> {
    static LOGOS: OnceLock<Vec<Arc<Image>>> = OnceLock::new();
    let name = name.to_lowercase();
    let maker = MAKERS
        .iter()
        .position(|(begins, _)| name.starts_with(begins))?;
    let logos = LOGOS.get_or_init(|| MAKERS.iter().map(|(_, png)| read(png)).collect());
    Some(logos[maker].clone())
}

/// The mark of row `index` of the integrations' list.
pub fn link(index: usize) -> Arc<Image> {
    static LOGOS: OnceLock<Vec<Arc<Image>>> = OnceLock::new();
    LOGOS.get_or_init(|| LINKS.iter().map(|png| read(png)).collect())[index].clone()
}

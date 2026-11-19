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

/// One for each row, in the lists' own order.
const MODELS: [&[u8]; steps::MODEL_LIST.len()] = [
    logo!("qwen"),
    logo!("mistral"),
    logo!("gemma"),
    logo!("microsoft"),
    logo!("hugging-face"),
    logo!("qwen"),
    logo!("openai"),
    logo!("qwen"),
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

/// The mark of row `index` of the models' list or the integrations'.
/// Each is read once and kept, so a row shows the same picture every time
/// it is drawn.
pub fn logo(step: usize, index: usize) -> Arc<Image> {
    static LOGOS: OnceLock<[Vec<Arc<Image>>; 2]> = OnceLock::new();
    let read = |pngs: &[&'static [u8]]| {
        pngs.iter()
            .map(|png| Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec())))
            .collect()
    };
    let logos = LOGOS.get_or_init(|| [read(&MODELS), read(&LINKS)]);
    logos[usize::from(step != steps::MODELS)][index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_picture_is_a_png() {
        for png in MODELS.iter().chain(&LINKS) {
            assert!(png.starts_with(b"\x89PNG"));
        }
    }
}

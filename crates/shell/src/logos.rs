// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The marks beside the rows of the models' and the connectors' lists,
//! in onboarding and in Settings: each in its 40-point box, at three
//! pixels to a point. Where they come from is in
//! `assets/onboarding/README.md`.

use std::sync::{Arc, OnceLock};

use ui::{Image, ImageFormat};

macro_rules! logos {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_bytes!(concat!("../../../assets/onboarding/logos/", $name, ".png")).as_slice()),)*]
    };
}

const LOGOS: [(&str, &[u8]); 34] = logos![
    "qwen",
    "mistral",
    "gemma",
    "microsoft",
    "hugging-face",
    "openai",
    "linear",
    "figma",
    "github",
    "zapier",
    "notion",
    "loom",
    "mailchimp",
    "dropbox",
    "jira",
    "confluence",
    "airtable",
    "sentry",
    "stripe",
    "cloudflare",
    "vercel",
    "supabase",
    "asana",
    "webflow",
    "intercom",
    "square",
    "paypal",
    "gmail",
    "google-drive",
    "google-docs",
    "google-calendar",
    "box",
    "hubspot",
    "zoom",
];

/// How a model's name begins when it is that maker's, and its mark.
const MAKERS: [(&str, &str); 6] = [
    ("qwen", "qwen"),
    ("mistral", "mistral"),
    ("gemma", "gemma"),
    ("phi", "microsoft"),
    ("smol", "hugging-face"),
    ("gpt", "openai"),
];

/// The logo kept as `name`. Each is read once and kept, so a row shows
/// the same picture every time it is drawn.
pub fn named(name: &str) -> Option<Arc<Image>> {
    static READ: OnceLock<Vec<Arc<Image>>> = OnceLock::new();
    let index = LOGOS.iter().position(|(kept, _)| *kept == name)?;
    let read = READ.get_or_init(|| {
        LOGOS
            .iter()
            .map(|(_, png)| Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec())))
            .collect()
    });
    Some(read[index].clone())
}

/// The mark of the maker of the model called `name`, or published by
/// `publisher`, where there is one.
pub fn model(name: &str, publisher: Option<&str>) -> Option<Arc<Image>> {
    let lower = |text: &str| text.to_lowercase().replace(' ', "-");
    let name = lower(name);
    let publisher = publisher.map(lower).unwrap_or_default();
    MAKERS
        .iter()
        .find(|(begins, mark)| name.starts_with(begins) || publisher.starts_with(mark))
        .and_then(|(_, mark)| named(mark))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalogue_logo_is_kept() {
        for connector in &crate::connectors::CONNECTORS {
            if let Some(logo) = connector.logo {
                assert!(named(logo).is_some(), "{logo}");
            }
        }
        assert!(model("Qwen3.5 4B", None).is_some());
        assert!(model("Chat Small", Some("Hugging Face")).is_some());
        assert!(model("Chat Small", None).is_none());
    }
}

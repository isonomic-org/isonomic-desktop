// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures on the Settings pages: icons from the library's set, by
//! name, and the marks the design shows beside a model, an app or a file.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use ui::assets::IconName;
use ui::component::Icon;
use ui::{Image, ImageFormat};

/// The library's icon called `name`, as its file is called; none when the
/// set has no such icon.
pub fn icon(name: &str) -> Icon {
    static BY_NAME: OnceLock<HashMap<String, IconName>> = OnceLock::new();
    let all = BY_NAME.get_or_init(|| {
        IconName::ALL
            .iter()
            .map(|icon| {
                let path = icon.path().to_string();
                let name = path.trim_start_matches("icons/").trim_end_matches(".svg");
                (name.to_string(), *icon)
            })
            .collect()
    });
    all.get(name)
        .map_or_else(Icon::empty, |icon| Icon::from(*icon))
}

macro_rules! marks {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_bytes!(concat!("../../../../assets/settings/logos/", $name, ".png")).as_slice()),)*]
    };
}

/// The marks only Settings shows. Where they come from is in
/// `assets/settings/README.md`.
const MARKS: [(&str, &[u8]); 12] = marks![
    "baai",
    "chrome",
    "deepseek",
    "excel",
    "google",
    "meta",
    "postgresql",
    "safari",
    "slack",
    "todoist",
    "wolfram",
    "word",
];

/// The mark kept as `name`: one the window's lists already have, or one of
/// Settings' own. Each is read once and kept, so a row shows the same
/// picture every time it is drawn.
pub fn logo(name: &str) -> Option<Arc<Image>> {
    static READ: OnceLock<Vec<Arc<Image>>> = OnceLock::new();
    if let Some(shared) = crate::logos::named(name) {
        return Some(shared);
    }
    let index = MARKS.iter().position(|(kept, _)| *kept == name)?;
    let read = READ.get_or_init(|| {
        MARKS
            .iter()
            .map(|(_, png)| Arc::new(Image::from_bytes(ImageFormat::Png, png.to_vec())))
            .collect()
    });
    Some(read[index].clone())
}

#[cfg(test)]
mod tests {
    use super::super::spec;
    use super::*;

    #[test]
    fn every_mark_a_row_names_is_kept() {
        for id in spec::ids() {
            let blocks = spec::spec(id).map_or(&[][..], |of| spec::list(of, "blocks"));
            for row in blocks.iter().flat_map(|block| spec::list(block, "rows")) {
                if let Some(name) = row["lead"]["logo"].as_str() {
                    assert!(logo(name).is_some(), "{id}: {name}");
                }
            }
        }
    }
}

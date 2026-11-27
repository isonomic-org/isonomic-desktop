// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which Mac this is, and the 3D model that stands for it on the
//! onboarding window: its own enclosure from the devices repository, in
//! the finish that enclosure is shown in.
//!
//! The Mac says its model identifier (`hw.model`). A Mac the table does
//! not know is shown as the newest enclosure of its line; one that names
//! no line (a virtual machine, a Mac newer than the table) is shown as the
//! design's laptop.

use super::machines::{Finish, MACS, MODELS};

/// The enclosure shown when nothing is known of this Mac: the design's
/// laptop.
const DESIGNS: &str = "macbook_pro13_2020";

/// How a line's identifiers begin, and the enclosure that stands for a
/// Mac of that line the table does not have. The longer beginnings come
/// first.
const LINES: [(&str, &str); 7] = [
    ("MacBookAir", "macbook_air13_2022"),
    ("MacBookPro", "macbook_pro14_2021"),
    ("MacBook", "macbook12_2015"),
    ("iMacPro", "imac27_2012"),
    ("iMac", "imac24_2021"),
    ("Macmini", "mac_mini_2024"),
    ("MacPro", "mac_pro_2019"),
];

/// Names another Mac's model identifier, to see its model on this one.
const STAND_IN: &str = "ISONOMIC_MAC_MODEL";

/// A Mac as onboarding shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Machine {
    pub enclosure: &'static str,
    /// The enclosure's 3D model, as its file.
    pub model: &'static [u8],
    /// The colours it is shown in, by group of the model's materials.
    pub finish: Finish,
}

/// This Mac's model identifier, such as `MacBookPro15,1`. It runs a
/// program and waits for it: keep it off the foreground thread.
pub fn identifier() -> Option<String> {
    if let Ok(named) = std::env::var(STAND_IN) {
        return Some(named);
    }
    let said = std::process::Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
        .ok()?;
    let identifier = String::from_utf8(said.stdout).ok()?;
    let identifier = identifier.trim();
    (said.status.success() && !identifier.is_empty()).then(|| identifier.to_string())
}

fn enclosure(named: &str) -> Machine {
    // The finish of the first Mac the table has in this enclosure.
    let known = MACS.iter().find(|(_, enclosure, _)| *enclosure == named);
    let model = MODELS.iter().find(|(enclosure, _)| *enclosure == named);
    match (known, model) {
        (Some((_, enclosure, finish)), Some((_, model))) => Machine {
            enclosure,
            model,
            finish,
        },
        // Every enclosure named here has both; the tests hold that.
        _ => Machine {
            enclosure: MODELS[0].0,
            model: MODELS[0].1,
            finish: &[],
        },
    }
}

/// The Mac with this model identifier, as onboarding shows it.
pub fn of(identifier: Option<&str>) -> Machine {
    let Some(identifier) = identifier else {
        return enclosure(DESIGNS);
    };
    if let Some((_, named, finish)) = MACS.iter().find(|(known, _, _)| *known == identifier) {
        return Machine {
            finish,
            ..enclosure(named)
        };
    }
    let line = LINES
        .iter()
        .find(|(begins, _)| identifier.starts_with(begins));
    enclosure(line.map_or(DESIGNS, |(_, named)| named))
}

/// Every enclosure there is a model of, as onboarding shows it.
pub fn every() -> impl Iterator<Item = Machine> {
    MODELS.iter().map(|(named, _)| enclosure(named))
}

/// This Mac, as onboarding shows it.
pub fn this_mac() -> Machine {
    of(identifier().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_mac_is_its_own_enclosure() {
        assert_eq!(of(Some("MacBookPro15,1")).enclosure, "macbook_pro15_2016");
        assert_eq!(of(Some("MacBookPro16,2")).enclosure, "macbook_pro13_2020");
        assert_eq!(of(Some("Macmini9,1")).enclosure, "mac_mini_2010");
        assert!(!of(Some("MacBookPro15,1")).finish.is_empty());
    }

    #[test]
    fn an_unknown_mac_is_its_line_s_newest_or_the_design_s_laptop() {
        assert_eq!(of(Some("MacBookAir99,1")).enclosure, "macbook_air13_2022");
        assert_eq!(of(Some("iMacPro9,9")).enclosure, "imac27_2012");
        assert_eq!(of(Some("VirtualMac2,1")).enclosure, DESIGNS);
        assert_eq!(of(Some("Mac99,9")).enclosure, DESIGNS);
        assert_eq!(of(None).enclosure, DESIGNS);
    }

    #[test]
    fn every_mac_in_the_table_has_a_model_that_reads() {
        for (identifier, named, _) in &MACS {
            let machine = of(Some(identifier));
            assert_eq!(machine.enclosure, *named, "{identifier}");
        }
        for (_, named) in LINES.iter().chain([&("", DESIGNS)]) {
            assert_eq!(enclosure(named).enclosure, *named);
        }
        for (named, model) in &MODELS {
            let read = device3d::Model::read(model);
            assert!(read.is_ok(), "{named}: {:?}", read.err());
            let groups = enclosure(named).finish;
            assert!(!groups.is_empty(), "{named} has a finish");
        }
    }
}

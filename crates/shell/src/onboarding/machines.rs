// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Macs the devices repository has a model of: each one's model
//! identifier, its enclosure, and the colours of the finish it is shown in.
//! Made from the repository's `assets/index.json` at 65e685e; the models are
//! its `assets/apple/<enclosure>/model.glb`, unchanged.

macro_rules! model {
    ($file:literal) => {
        include_bytes!(concat!("../../../../assets/onboarding/", $file, ".glb")).as_slice()
    };
}

/// A finish: the colour of each group of a model's materials.
pub type Finish = &'static [(&'static str, [u8; 3])];

/// Each enclosure's 3D model.
pub const MODELS: [(&str, &[u8]); 23] = [
    ("imac21_2012", model!("macs/imac21-2012")),
    ("imac24_2021", model!("macs/imac24-2021")),
    ("imac27_2012", model!("macs/imac27-2012")),
    ("mac_mini_2010", model!("macs/mac-mini-2010")),
    ("mac_mini_2024", model!("macs/mac-mini-2024")),
    ("mac_pro_2013", model!("macs/mac-pro-2013")),
    ("mac_pro_2019", model!("macs/mac-pro-2019")),
    ("mac_studio_2022", model!("macs/mac-studio-2022")),
    ("macbook12_2015", model!("macs/macbook12-2015")),
    ("macbook_air13_2010", model!("macs/macbook-air13-2010")),
    ("macbook_air13_2018", model!("macs/macbook-air13-2018")),
    ("macbook_air13_2020", model!("macs/macbook-air13-2020")),
    ("macbook_air13_2022", model!("macs/macbook-air13-2022")),
    ("macbook_air15_2023", model!("macs/macbook-air15-2023")),
    (
        "macbook_pro13_2016_fn",
        model!("macs/macbook-pro13-2016-fn"),
    ),
    (
        "macbook_pro13_2016_tb",
        model!("macs/macbook-pro13-2016-tb"),
    ),
    ("macbook_pro13_2020", model!("macbook-pro-13-2020")),
    ("macbook_pro14_2021", model!("macs/macbook-pro14-2021")),
    (
        "macbook_pro14_2026_oled",
        model!("macs/macbook-pro14-2026-oled"),
    ),
    ("macbook_pro15_2016", model!("macs/macbook-pro15-2016")),
    ("macbook_pro16_2019", model!("macs/macbook-pro16-2019")),
    ("macbook_pro16_2021", model!("macs/macbook-pro16-2021")),
    (
        "macbook_pro16_2026_oled",
        model!("macs/macbook-pro16-2026-oled"),
    ),
];

const F0: Finish = &[("body", [0xCD, 0xCE, 0xD1])];
const F1: Finish = &[("body", [0x80, 0x81, 0x84])];
const F2: Finish = &[("body", [0xE7, 0xD5, 0xC4])];
const F3: Finish = &[("body", [0xD9, 0xDA, 0xDC]), ("chin", [0xE9, 0xE9, 0xEA])];
const F4: Finish = &[("body", [0xD9, 0xDA, 0xDC])];
const F5: Finish = &[("body", [0x1C, 0x1C, 0x1F])];

/// A Mac by its model identifier: its enclosure and its finish.
pub const MACS: [(&str, &str, Finish); 100] = [
    ("Mac18,1", "macbook_pro14_2021", F0),
    ("Mac17,11", "macbook_pro14_2026_oled", F0),
    ("Mac17,13", "macbook_pro14_2026_oled", F0),
    ("Mac17,10", "macbook_pro16_2026_oled", F0),
    ("Mac17,12", "macbook_pro16_2026_oled", F0),
    ("Mac17,7", "macbook_pro14_2021", F0),
    ("Mac17,9", "macbook_pro14_2021", F0),
    ("Mac17,6", "macbook_pro16_2021", F0),
    ("Mac17,8", "macbook_pro16_2021", F0),
    ("Mac17,2", "macbook_pro14_2021", F0),
    ("Mac16,1", "macbook_pro14_2021", F0),
    ("Mac16,6", "macbook_pro14_2021", F0),
    ("Mac16,8", "macbook_pro14_2021", F0),
    ("Mac16,7", "macbook_pro16_2021", F0),
    ("Mac16,5", "macbook_pro16_2021", F0),
    ("Mac15,3", "macbook_pro14_2021", F0),
    ("Mac15,6", "macbook_pro14_2021", F0),
    ("Mac15,8", "macbook_pro14_2021", F0),
    ("Mac15,10", "macbook_pro14_2021", F0),
    ("Mac15,7", "macbook_pro16_2021", F0),
    ("Mac15,9", "macbook_pro16_2021", F0),
    ("Mac15,11", "macbook_pro16_2021", F0),
    ("Mac14,5", "macbook_pro14_2021", F0),
    ("Mac14,9", "macbook_pro14_2021", F0),
    ("Mac14,6", "macbook_pro16_2021", F0),
    ("Mac14,10", "macbook_pro16_2021", F0),
    ("Mac14,7", "macbook_pro13_2020", F0),
    ("MacBookPro18,3", "macbook_pro14_2021", F0),
    ("MacBookPro18,4", "macbook_pro14_2021", F0),
    ("MacBookPro18,1", "macbook_pro16_2021", F0),
    ("MacBookPro18,2", "macbook_pro16_2021", F0),
    ("MacBookPro17,1", "macbook_pro13_2020", F0),
    ("MacBookPro16,3", "macbook_pro13_2020", F0),
    ("MacBookPro16,2", "macbook_pro13_2020", F0),
    ("MacBookPro16,1", "macbook_pro16_2019", F0),
    ("MacBookPro16,4", "macbook_pro16_2019", F0),
    ("MacBookPro15,4", "macbook_pro13_2016_tb", F0),
    ("MacBookPro15,1", "macbook_pro15_2016", F0),
    ("MacBookPro15,3", "macbook_pro15_2016", F0),
    ("MacBookPro15,2", "macbook_pro13_2016_tb", F0),
    ("MacBookPro14,3", "macbook_pro15_2016", F0),
    ("MacBookPro14,2", "macbook_pro13_2016_tb", F0),
    ("MacBookPro14,1", "macbook_pro13_2016_fn", F0),
    ("MacBookPro13,3", "macbook_pro15_2016", F0),
    ("MacBookPro13,2", "macbook_pro13_2016_tb", F0),
    ("MacBookPro13,1", "macbook_pro13_2016_fn", F0),
    ("Mac17,4", "macbook_air15_2023", F0),
    ("Mac17,3", "macbook_air13_2022", F0),
    ("Mac16,13", "macbook_air15_2023", F0),
    ("Mac16,12", "macbook_air13_2022", F0),
    ("Mac15,13", "macbook_air15_2023", F0),
    ("Mac15,12", "macbook_air13_2022", F0),
    ("Mac14,15", "macbook_air15_2023", F0),
    ("Mac14,2", "macbook_air13_2022", F0),
    ("MacBookAir10,1", "macbook_air13_2020", F1),
    ("MacBookAir9,1", "macbook_air13_2020", F1),
    ("MacBookAir8,2", "macbook_air13_2018", F1),
    ("MacBookAir8,1", "macbook_air13_2018", F1),
    ("MacBookAir7,2", "macbook_air13_2010", F0),
    ("MacBook10,1", "macbook12_2015", F2),
    ("MacBook9,1", "macbook12_2015", F2),
    ("Mac18,2", "imac24_2021", F3),
    ("Mac16,3", "imac24_2021", F3),
    ("Mac16,2", "imac24_2021", F3),
    ("Mac15,5", "imac24_2021", F3),
    ("Mac15,4", "imac24_2021", F3),
    ("iMac21,1", "imac24_2021", F3),
    ("iMac21,2", "imac24_2021", F3),
    ("iMac20,1", "imac27_2012", F0),
    ("iMac20,2", "imac27_2012", F0),
    ("iMac19,1", "imac27_2012", F0),
    ("iMac19,2", "imac21_2012", F0),
    ("iMacPro1,1", "imac27_2012", F1),
    ("iMac18,3", "imac27_2012", F0),
    ("iMac18,2", "imac21_2012", F0),
    ("iMac18,1", "imac21_2012", F0),
    ("iMac17,1", "imac27_2012", F0),
    ("iMac16,2", "imac21_2012", F0),
    ("iMac16,1", "imac21_2012", F0),
    ("iMac15,1", "imac27_2012", F0),
    ("Mac18,5", "mac_mini_2024", F0),
    ("Mac17,16", "mac_mini_2024", F0),
    ("Mac16,10", "mac_mini_2024", F0),
    ("Mac16,11", "mac_mini_2024", F0),
    ("Mac14,3", "mac_mini_2010", F0),
    ("Mac14,12", "mac_mini_2010", F0),
    ("Macmini9,1", "mac_mini_2010", F0),
    ("Macmini8,1", "mac_mini_2010", F1),
    ("Macmini7,1", "mac_mini_2010", F0),
    ("Mac17,14", "mac_studio_2022", F0),
    ("Mac17,15", "mac_studio_2022", F0),
    ("Mac16,9", "mac_studio_2022", F0),
    ("Mac15,14", "mac_studio_2022", F0),
    ("Mac14,13", "mac_studio_2022", F0),
    ("Mac14,14", "mac_studio_2022", F0),
    ("Mac13,1", "mac_studio_2022", F0),
    ("Mac13,2", "mac_studio_2022", F0),
    ("Mac14,8", "mac_pro_2019", F4),
    ("MacPro7,1", "mac_pro_2019", F4),
    ("MacPro6,1", "mac_pro_2013", F5),
];

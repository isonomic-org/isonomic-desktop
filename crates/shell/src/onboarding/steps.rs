// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words of onboarding's later steps, from the design ("Onboarding
//! 0.4"). The ID, the kit's words and the models shown are the backend's;
//! the samples kept here (`WORDS`, `MODEL_LIST`) are what
//! `backend::Sample` answers with, for tests and drawings.

/// A step: what the stepper says for it, what its panel is headed, and
/// its two footer buttons.
pub struct Step {
    pub id: &'static str,
    pub label: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    /// How many of the stepper's bars are filled.
    pub bars: usize,
    pub heading: &'static str,
    pub lead: &'static str,
    /// Is the heading set off from the body by a rule?
    pub ruled: bool,
    pub forward: &'static str,
}

pub const CREATED: usize = 0;
pub const KIT: usize = 1;
pub const CONFIRM: usize = 2;
pub const MODELS: usize = 3;
pub const LINKS: usize = 4;
pub const FINISH: usize = 5;

const ACCOUNT: (&str, &str, &str, usize) = (
    "STEP 2",
    "Create account",
    "Face ID creates your account. No seed phrase, no password.",
    2,
);
const RECOVERY: (&str, &str, &str, usize) = (
    "STEP 3",
    "Recovery kit",
    "Save the kit that protects your encrypted data if this device is lost.",
    3,
);

const fn step(
    id: &'static str,
    (label, name, about, bars): (&'static str, &'static str, &'static str, usize),
    heading: &'static str,
    lead: &'static str,
    ruled: bool,
    forward: &'static str,
) -> Step {
    Step {
        id,
        label,
        name,
        about,
        bars,
        heading,
        lead,
        ruled,
        forward,
    }
}

pub const STEPS: [Step; 6] = [
    step(
        "onboarding-created",
        ACCOUNT,
        "Your account exists.",
        "The passkey is in your secure enclave and already syncing to your other Apple devices. Here is the address it derives.",
        false,
        "Continue",
    ),
    step(
        "onboarding-kit",
        RECOVERY,
        "Save your recovery kit",
        "These 24 words and the kit file together bring your Isonomic ID back on another computer. Write the words down and save the file somewhere else: neither works alone.",
        true,
        "Continue",
    ),
    step(
        "onboarding-confirm",
        RECOVERY,
        "Confirm your recovery kit",
        "Type words 4, 11 and 19 from your kit to confirm you wrote them down correctly. Capital letters and surrounding spaces don\u{2019}t matter.",
        true,
        "Continue",
    ),
    step(
        "onboarding-models",
        (
            "STEP 4 \u{b7} OPTIONAL",
            "Local AI models",
            "Choose which models run on this machine. You can change this any time.",
            4,
        ),
        "Turn on local AI models",
        "Models run on this machine and start only when an app needs them.",
        true,
        "Next step",
    ),
    step(
        "onboarding-links",
        (
            "STEP 5 \u{b7} OPTIONAL",
            "Add integrations",
            "Supercharge your workflow and connect the tools you use every day to your account.",
            5,
        ),
        "Integrations and connected apps",
        "Supercharge your workflow and connect the tool you use every day.",
        true,
        "Next step",
    ),
    step(
        "onboarding-finish",
        (
            "STEP 6",
            "Finish",
            "Set autostart and menu bar options, then launch.",
            6,
        ),
        "Isonomic lives in your menu bar",
        "Closing a window hides it \u{2014} your node keeps running. Only Quit from the menu bar shuts the stack down, in reverse dependency order.",
        false,
        "Finish setup",
    ),
];

pub const ADDRESS_NOTE: &str = "Made on this Mac, with its keys in this Mac\u{2019}s key store. It is yours in every Isonomic app, and your recovery kit carries it to another computer.";
pub const ROOT: [(&str, &str); 4] = [
    ("App licences & subscriptions", "ERC-1155 on Base"),
    ("Node registration", "NodeRegistry on Isonomic L2"),
    (
        "Identity header injected into apps",
        "X-Isonomic-Identity JWT",
    ),
    ("Workspace encryption", "WMK wrapped by the passkey PRF"),
];

pub const KIT_WARNING: (&str, &str) = (
    "Keep the words and the kit file apart",
    "Anyone holding both can take your ID. The file alone can\u{2019}t, and Isonomic never saves the words: write them down now. Restoring on another computer spends this kit and gives you a new one.",
);
pub const WORDS: [&str; 24] = [
    "orbit", "crane", "velvet", "hollow", "ladder", "survey", "bamboo", "glide", "ancient",
    "pepper", "ribbon", "fossil", "timber", "quantum", "mixture", "harbor", "jewel", "lunar",
    "sketch", "brave", "cotton", "empower", "ripple", "oxygen",
];
/// How many words a kit has.
pub const WORD_COUNT: usize = 24;
/// The words asked back, by their place in the kit.
pub const ASKED: [usize; 3] = [4, 11, 19];
pub const CONFIRM_CAPTION: &str = "ISONOMIC RECOVERY KIT \u{b7} ENTER WORDS 4, 11 AND 19";
pub const CONFIRM_NOTE: (&str, &str) = (
    "Check against your written copy, not your memory",
    "Look at the copy you wrote down or engraved. If a word doesn\u{2019}t match, go back and write the kit down again before you continue.",
);

/// Each sample model: its name, its publisher, what it is for, its
/// kind, whether the engine would start with it, and whether it fits
/// this Mac.
#[rustfmt::skip]
pub const MODEL_LIST: [(&str, &str, &str, &str, bool, bool); 8] = [
    ("Qwen3.5 4B", "Qwen", "Default chat model on every tier.", "Chat", true, true),
    ("Mistral Small 4", "Mistral AI", "Chat model for the Mac Studio tier.", "Chat", false, true),
    ("Gemma 4 E4B", "Google", "It can also take images and audio later.", "Multimodal", false, true),
    ("Phi-4-mini", "Microsoft", "Small MIT-licensed fallback.", "Utility", false, true),
    ("SmolLM3 3B", "Hugging Face", "Fully open, training data included.", "Utility", false, true),
    ("Qwen3.5 0.8B", "Qwen", "Tiny utility model for titles, routing and summaries.", "Utility", true, true),
    ("gpt-oss-20b", "OpenAI", "Reasoning and tool use on Apple Silicon.", "Reasoning", false, true),
    ("Qwen3.6 35B-A3B", "Qwen", "The flagship for Apple Silicon with 32 GB or more.", "Chat", false, false),
];

/// What a sample model's pill says when its "i" is pointed at.
pub fn fit_reason(fits: bool) -> &'static str {
    if fits {
        "It fits in this Mac\u{2019}s 16 GB of memory, with room for your apps."
    } else {
        "It needs about 24 GB of memory, and this Mac has 16 GB."
    }
}

/// An ID cut short for a line of small print: its start and its end.
pub fn short(id: &str) -> String {
    let letters: Vec<char> = id.chars().collect();
    if letters.len() <= 28 {
        return id.to_owned();
    }
    let (start, end) = (&letters[..16], &letters[letters.len() - 6..]);
    format!(
        "{}\u{2026}{}",
        start.iter().collect::<String>(),
        end.iter().collect::<String>()
    )
}

pub const RESIDENCY: [(&str, &str, bool); 3] = [
    (
        "Start Isonomic at login",
        "Boots minimised to the menu bar. Tier 0\u{2013}1 services start, no windows open, the node begins heartbeating.",
        true,
    ),
    (
        "Allow AI web access",
        "Off means prompts never leave this machine \u{2014} the router hard-stops at local-only. You can flip this from the menu bar at any time.",
        false,
    ),
    (
        "Nightly encrypted snapshots",
        "pg_dump of each app database into the system-backups bucket, encrypted with your workspace key.",
        true,
    ),
];
pub const AFTER: (&str, &str) = (
    "What happens when you finish",
    "Caddy and SQLite come up first, then Postgres, Valkey and SeaweedFS in parallel. Target is all healthy in under 10 seconds. You land on the launcher with an empty apps grid and a link to the store.",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_id_is_cut_short_and_a_short_one_is_not() {
        let long = "did:iso:c6eeabb1d5d2823a365417297615ace4cf5dc74d1ea9f6cfa261b63b77b75353";
        assert_eq!(short(long), "did:iso:c6eeabb1\u{2026}b75353");
        assert_eq!(short("isonomic:7Q4K-M2XD-9PLV"), "isonomic:7Q4K-M2XD-9PLV");
    }
}

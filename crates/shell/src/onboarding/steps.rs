// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words of onboarding's later steps, from the design ("Onboarding
//! 0.4"). This is a first pass with nothing behind it: every value here
//! is the design's sample, not this machine's.

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
    "STEP 1",
    "Create account",
    "Face ID creates your account. No seed phrase, no password.",
    1,
);
const RECOVERY: (&str, &str, &str, usize) = (
    "STEP 2",
    "Recovery kit",
    "Save the kit that protects your encrypted data if this device is lost.",
    2,
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
        "These 24 words are a data-recovery key, not an account seed. Your account is recoverable through your passkey; your encrypted data is not \u{2014} unless you keep these words.",
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
            "STEP 3 \u{b7} OPTIONAL",
            "Local AI models",
            "Choose which models run on this machine. You can change this any time.",
            3,
        ),
        "Turn on local AI models",
        "Models run on this machine and start only when an app needs them.",
        true,
        "Next step",
    ),
    step(
        "onboarding-links",
        (
            "STEP 4 \u{b7} OPTIONAL",
            "Add integrations",
            "Supercharge your workflow and connect the tools you use every day to your account.",
            4,
        ),
        "Integrations and connected apps",
        "Supercharge your workflow and connect the tool you use every day.",
        true,
        "Next step",
    ),
    step(
        "onboarding-finish",
        (
            "STEP 5",
            "Finish",
            "Set autostart and menu bar options, then launch.",
            5,
        ),
        "Isonomic lives in your menu bar",
        "Closing a window hides it \u{2014} your node keeps running. Only Quit from the menu bar shuts the stack down, in reverse dependency order.",
        false,
        "Finish setup",
    ),
];

pub const ADDRESS: &str = "0x7A3f2b19E4c8D05fAe61B7d3C42a90118Be59C21";
pub const ADDRESS_NOTE: &str = "Counterfactual \u{2014} the contract deploys on your first transaction, with gas sponsored by the paymaster. Same address on Base and the Isonomic L2.";
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
    "Account recovery and data recovery are different problems",
    "Move to a new Apple device and your passkey follows. Lose your platform account and the account is recoverable through guardian modules \u{2014} but the workspace key is not. Without these words, that data is gone.",
);
pub const WORDS: [&str; 24] = [
    "orbit", "crane", "velvet", "hollow", "ladder", "survey", "bamboo", "glide", "ancient",
    "pepper", "ribbon", "fossil", "timber", "quantum", "mixture", "harbor", "jewel", "lunar",
    "sketch", "brave", "cotton", "empower", "ripple", "oxygen",
];
pub const KIT_NOTE: &str =
    "Generated 2 Aug 2026 \u{b7} Account 0x7A3f\u{2026}9C21 \u{b7} Store it away from this machine";
/// The words asked back, by their place in the kit.
pub const ASKED: [usize; 3] = [4, 11, 19];
pub const CONFIRM_CAPTION: &str = "ISONOMIC RECOVERY KIT \u{b7} ENTER WORDS 4, 11 AND 19";
pub const CONFIRM_NOTE: (&str, &str) = (
    "Check against your saved kit, not your memory",
    "Look at the copy you printed, saved or engraved. If a word doesn\u{2019}t match, go back and save the kit again before you continue.",
);

pub const MODEL_TABS: [&str; 5] = ["All models", "Chat", "Reasoning", "Multimodal", "Utility"];
/// Each model, what it is for, and whether it starts switched on.
pub const MODEL_LIST: [(&str, &str, bool); 8] = [
    ("Qwen3.5 4B", "Default chat model on every tier.", true),
    (
        "Mistral Small 4",
        "Chat model for the Mac Studio tier.",
        false,
    ),
    (
        "Gemma 4 E4B",
        "A second model family. It can also take images and audio later.",
        false,
    ),
    ("Phi-4-mini", "Small MIT-licensed fallback.", false),
    ("SmolLM3 3B", "Fully open, training data included.", false),
    (
        "Qwen3.5 0.8B",
        "Tiny utility model for titles, routing and summaries.",
        true,
    ),
    (
        "gpt-oss-20b",
        "Reasoning and tool use on Apple Silicon.",
        false,
    ),
    (
        "Qwen3.6 35B-A3B",
        "The flagship for Apple Silicon with 32 GB or more.",
        false,
    ),
];

pub const LINK_TABS: [&str; 5] = [
    "All integrations",
    "Developer tools",
    "Communication",
    "Productivity",
    "Browser tools",
];
pub const LINK_LIST: [(&str, &str, bool); 8] = [
    (
        "Linear",
        "Streamline software projects, sprints, tasks, and bug tracking.",
        true,
    ),
    ("Figma", "Embed file previews in projects.", true),
    (
        "GitHub",
        "Link pull requests and automate workflows.",
        false,
    ),
    (
        "Zapier",
        "Build custom automations and integrations with other apps.",
        false,
    ),
    ("Notion", "Embed notion pages and notes in projects.", false),
    (
        "Loom",
        "Boost your team\u{2019}s productivity with async video collaboration and recording.",
        false,
    ),
    (
        "Mailchimp",
        "Grow your business an all-in-one marketing & email marketing platform.",
        false,
    ),
    (
        "Dropbox",
        "Seamlessly connect your team\u{2019}s cloud files and storage with current projects.",
        false,
    ),
];

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

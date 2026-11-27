// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What onboarding and Settings ask of the engine about the person's
//! workspace: the local models this Mac could run, their downloads and
//! removal; and the tool servers connected, their sign-ins and their
//! changes.
//!
//! The window's is [`engine::EngineWorkspace`], over the shell's client.
//! Every refusal is plain words, the engine's own where it has them. A
//! connection's secret is never logged.

pub mod engine;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_proto::connections::{AddConnection, Connection};
use isonomic_proto::models::DownloadProgress;

pub type Answer<T> = BoxFuture<'static, Result<T, String>>;

/// The first model tab, which shows every model.
pub const ALL_MODELS: &str = "All models";

/// A local model, as its row shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub name: String,
    /// Who publishes it, and one line about it.
    pub publisher: Option<String>,
    pub about: String,
    /// The tab it is under: what it is for.
    pub kind: String,
    pub size_bytes: Option<u64>,
    /// Is it the one the engine would start with?
    pub first: bool,
    pub installed: bool,
    pub downloading: bool,
    /// Does this Mac's memory admit it, and why, in a sentence or two.
    pub fits: bool,
    pub fit_reason: Option<String>,
}

impl Model {
    /// The line under its name: its publisher, what it is, and its size.
    pub fn line(&self) -> String {
        let size = if self.installed {
            Some("Installed".to_owned())
        } else {
            self.size_bytes
                .map(|bytes| format!("{} download", size(bytes)))
        };
        let parts = [self.publisher.clone(), Some(self.about.clone()), size];
        let parts: Vec<String> = parts
            .into_iter()
            .flatten()
            .filter(|part| !part.is_empty())
            .collect();
        parts.join(" \u{b7} ")
    }

    /// Its pill's words.
    pub fn pill(&self) -> &'static str {
        if self.fits {
            "Recommended"
        } else {
            "Not recommended"
        }
    }

    /// What hovering the pill's "i" says.
    pub fn reason(&self) -> String {
        self.fit_reason.clone().unwrap_or_else(|| {
            if self.fits {
                "This Mac has the memory it needs.".to_owned()
            } else {
                "This Mac may not have the memory it needs.".to_owned()
            }
        })
    }
}

/// Has this Mac the memory local models need?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Memory {
    Enough,
    Short { has: u64, needs: u64 },
}

/// The local models this Mac could run, and its tabs, from them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub models: Vec<Model>,
    pub memory: Memory,
}

/// The order the kinds of model are tabbed in, when a catalogue has them.
const KINDS: [&str; 5] = ["Chat", "Reasoning", "Multimodal", "Embeddings", "Utility"];

impl Catalog {
    /// "All models", then each kind the catalogue has.
    pub fn tabs(&self) -> Vec<String> {
        let mut tabs = vec![ALL_MODELS.to_owned()];
        let mut kinds: Vec<&str> = self.models.iter().map(|m| m.kind.as_str()).collect();
        kinds.sort_by_key(|kind| KINDS.iter().position(|k| k == kind).unwrap_or(KINDS.len()));
        kinds.dedup();
        tabs.extend(kinds.into_iter().map(str::to_owned));
        tabs
    }

    /// Is `model` under tab `tab`, with `sought` in the search box?
    pub fn listed(&self, model: &Model, tab: usize, sought: &str) -> bool {
        let sought = sought.trim().to_lowercase();
        let under = tab == 0 || self.tabs().get(tab) == Some(&model.kind);
        let says = |text: &str| text.to_lowercase().contains(&sought);
        under && (says(&model.name) || says(&model.line()))
    }
}

/// A size as a person reads it: "2.4 GB", "640 MB".
pub fn size(bytes: u64) -> String {
    const GB: f64 = 1_000_000_000.;
    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else {
        format!("{:.0} MB", bytes / 1_000_000.)
    }
}

pub trait Workspace: Send + Sync + 'static {
    /// Every local model this Mac could install or has installed.
    fn catalogue(&self) -> Answer<Catalog>;
    /// Download `model`, or follow its running download. The stream ends
    /// after the event that has an outcome; dropping it only stops
    /// following.
    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>>;
    /// Delete an installed model's file.
    fn remove_model(&self, model: &str) -> Answer<()>;
    /// The tool servers connected.
    fn connections(&self) -> Answer<Vec<Connection>>;
    /// Connect one. Its secret, if any, is never logged.
    fn connect(&self, add: AddConnection) -> Answer<Connection>;
    /// Start its browser sign-in: the page to open.
    fn sign_in(&self, id: &str) -> Answer<String>;
    fn disconnect(&self, id: &str) -> Answer<()>;
    /// Each connection as it changes, from now on.
    fn changes(&self) -> BoxStream<'static, Connection>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(kind: &str, fits: bool) -> Model {
        Model {
            id: kind.to_lowercase(),
            name: format!("{kind} Model"),
            publisher: Some("Maker".into()),
            about: "A line.".into(),
            kind: kind.into(),
            size_bytes: Some(2_400_000_000),
            first: false,
            installed: false,
            downloading: false,
            fits,
            fit_reason: None,
        }
    }

    #[test]
    fn the_tabs_are_the_kinds_the_catalogue_has() {
        let catalog = Catalog {
            models: vec![
                model("Utility", true),
                model("Chat", true),
                model("Chat", false),
            ],
            memory: Memory::Enough,
        };
        assert_eq!(catalog.tabs(), [ALL_MODELS, "Chat", "Utility"]);
        assert!(catalog.listed(&catalog.models[0], 2, "maker"));
        assert!(!catalog.listed(&catalog.models[0], 1, ""));
        assert_eq!(
            catalog.models[0].line(),
            "Maker \u{b7} A line. \u{b7} 2.4 GB download"
        );
        assert_eq!(catalog.models[2].pill(), "Not recommended");
    }

    #[test]
    fn sizes_read_as_a_person_says_them() {
        assert_eq!(size(2_400_000_000), "2.4 GB");
        assert_eq!(size(640_000_000), "640 MB");
    }
}

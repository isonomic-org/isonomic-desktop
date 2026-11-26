// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding over the engine: the Isonomic ID and its recovery kit, the
//! local models and their downloads, through the shell's client (the
//! engine lists only the models a Mac has, so a new Mac is shown the one
//! the engine would start with); the
//! choices kept in the Capsule's settings; and "Start Isonomic at login"
//! through the login agent, where this build has one.
//!
//! A call waits for the engine to be connected: it is still starting
//! while the first screens play. Whoever calls gives up after its own
//! wait.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::stream::BoxStream;
use isonomic_app::SettingsStore;
use isonomic_client::{Client, ClientError};
use isonomic_proto::models::{DownloadProgress, ModelDetail, ModelState};
use isonomic_proto::ErrorCode;

use super::backend::{
    kept, off_thread, Account, Answer, Backend, Catalog, Choices, Kit, Memory, Model,
};
use crate::menubar::LoginToggle;

/// Where the window's own settings are kept, beside the apps'.
const SETTINGS: &str = "isonomic";
/// The terms the person accepts, by the day they were last changed.
const TERMS: &str = "2026-11-25";

pub struct EngineBackend {
    client: Client,
    settings: Arc<dyn SettingsStore>,
    login: Option<Arc<dyn LoginToggle>>,
}

impl EngineBackend {
    pub fn new(
        client: Client,
        settings: Arc<dyn SettingsStore>,
        login: Option<Arc<dyn LoginToggle>>,
    ) -> Self {
        Self {
            client,
            settings,
            login,
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// An error in the words the person reads. The engine's own are shown as
/// written.
fn said(error: ClientError) -> String {
    match error {
        ClientError::Engine(wire) if wire.code == ErrorCode::UnknownMethod => {
            "This copy of Isonomic\u{2019}s engine is older than its window. Update Isonomic and try again."
                .to_owned()
        }
        ClientError::Engine(wire) => wire.message,
        ClientError::Disconnected => {
            "Isonomic\u{2019}s engine isn\u{2019}t running. Try again in a moment.".to_owned()
        }
        other => format!("Isonomic\u{2019}s engine didn\u{2019}t answer as expected ({other})."),
    }
}

async fn connected(client: &Client) {
    let mut live = client.live();
    let _ = live.wait_for(|live| live.connected).await;
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

/// The tab a model is under, from what it can do.
fn kind(detail: &ModelDetail) -> usize {
    let can = |what: &str| detail.info.supports(what);
    if can("vision") || can("audio") || can("image") {
        3
    } else if can("reasoning") {
        2
    } else if can("chat") {
        1
    } else {
        4
    }
}

fn row(detail: ModelDetail) -> Model {
    let installed = !matches!(
        detail.state,
        ModelState::NotInstalled | ModelState::Downloading
    );
    let mut about = Vec::new();
    if detail.info.recommended {
        about.push("Recommended for this Mac".to_owned());
    }
    about.push(if installed {
        "Installed".to_owned()
    } else {
        detail.size_bytes.map_or_else(
            || "Download".to_owned(),
            |bytes| format!("{} download", size(bytes)),
        )
    });
    about.extend(detail.license.clone());
    Model {
        kind: kind(&detail),
        on: installed || detail.info.recommended,
        installed,
        about: about.join(" \u{b7} "),
        name: conversation_name(&detail.info.name),
        id: detail.info.id,
    }
}

/// A model's name without its quantisation tag, as Chat names it.
fn conversation_name(name: &str) -> String {
    name.split(" (").next().unwrap_or(name).trim().to_owned()
}

impl Backend for EngineBackend {
    fn accept_terms(&self) -> Answer<()> {
        let settings = Arc::clone(&self.settings);
        off_thread(move || {
            let accepted = serde_json::json!({ "version": TERMS, "accepted_at": now() });
            settings.set(SETTINGS, "terms", accepted);
            Ok(())
        })
    }

    fn create_account(&self) -> Answer<Account> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            let identity = client.identity().await.map_err(said)?;
            Ok(Account {
                id: identity.id,
                created: identity.created,
            })
        })
    }

    fn recovery_kit(&self) -> Answer<Kit> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            let kit = client.kit_export(true).await.map_err(said)?;
            Ok(Kit {
                id: kit.id,
                words: kit.words,
                document: kit.kit,
            })
        })
    }

    fn kit_saved(&self) -> Answer<()> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            client.kit_saved().await.map_err(said)
        })
    }

    fn models(&self) -> Answer<Catalog> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            let models = client.models();
            // The engine lists the models this Mac has. The one it would
            // start with is asked for by name: on a new Mac it is the
            // only one there is to show.
            let first = models.recommended(None).await;
            let memory = match &first {
                Err(ClientError::Engine(wire)) if wire.code == ErrorCode::InsufficientMemory => {
                    wire.memory
                        .as_ref()
                        .map_or(Memory::Enough, |refusal| Memory::Short {
                            has: refusal.available_bytes,
                            needs: refusal.required_bytes,
                        })
                }
                _ => Memory::Enough,
            };
            let mut ids: Vec<String> = first.into_iter().map(|first| first.model).collect();
            for info in models.list().await.map_err(said)? {
                if !info.remote && info.supports("chat") && !ids.contains(&info.id) {
                    ids.push(info.id);
                }
            }
            let mut rows = Vec::new();
            for id in ids {
                rows.push(row(models.info(&id).await.map_err(said)?));
            }
            Ok(Catalog {
                models: rows,
                memory,
            })
        })
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        crate::downloads::download(&self.client, model)
    }

    fn starts_at_login(&self) -> Option<bool> {
        self.login.as_ref().map(|login| login.enabled())
    }

    fn finish(&self, choices: Choices) -> Answer<()> {
        let (settings, login) = (Arc::clone(&self.settings), self.login.clone());
        off_thread(move || {
            let mut kept = kept(&choices);
            kept["finished_at"] = now().into();
            settings.set(SETTINGS, "onboarding", kept);
            match login {
                Some(login) if login.enabled() != choices.start_at_login => {
                    login.set_enabled(choices.start_at_login)
                }
                _ => Ok(()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_as_a_person_says_them() {
        assert_eq!(size(2_400_000_000), "2.4 GB");
        assert_eq!(size(640_000_000), "640 MB");
    }

    #[test]
    fn a_name_loses_its_quantisation_tag() {
        assert_eq!(conversation_name("Chat Small (Q4_K_M)"), "Chat Small");
        assert_eq!(conversation_name("Chat Small"), "Chat Small");
    }
}

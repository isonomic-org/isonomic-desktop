// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding over the engine: the Isonomic ID and its recovery kit
//! through the shell's client, and the models and connections through
//! the workspace's (`crate::workspace::engine`); the choices kept in the Capsule's settings; and "Start Isonomic at login"
//! through the login agent, where this build has one.
//!
//! A call waits for the engine to be connected: it is still starting
//! while the first screens play. Whoever calls gives up after its own
//! wait.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::stream::BoxStream;
use isonomic_app::SettingsStore;
use isonomic_client::Client;
use isonomic_proto::connections::{AddConnection, Connection};
use isonomic_proto::models::DownloadProgress;

use super::backend::{kept, off_thread, Account, Answer, Backend, Catalog, Choices, Kit};
use crate::menubar::LoginToggle;
use crate::workspace::engine::{connected, said, EngineWorkspace};
use crate::workspace::Workspace;

/// Where the window's own settings are kept, beside the apps'.
const SETTINGS: &str = "isonomic";
/// The terms the person accepts, by the day they were last changed.
const TERMS: &str = "2026-11-25";

pub struct EngineBackend {
    client: Client,
    workspace: EngineWorkspace,
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
            workspace: EngineWorkspace::new(client.clone()),
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

/// The models and connections are the workspace's.
impl Workspace for EngineBackend {
    fn catalogue(&self) -> Answer<Catalog> {
        self.workspace.catalogue()
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        self.workspace.download(model)
    }

    fn remove_model(&self, model: &str) -> Answer<()> {
        self.workspace.remove_model(model)
    }

    fn connections(&self) -> Answer<Vec<Connection>> {
        self.workspace.connections()
    }

    fn connect(&self, add: AddConnection) -> Answer<Connection> {
        self.workspace.connect(add)
    }

    fn sign_in(&self, id: &str) -> Answer<String> {
        self.workspace.sign_in(id)
    }

    fn disconnect(&self, id: &str) -> Answer<()> {
        self.workspace.disconnect(id)
    }

    fn changes(&self) -> BoxStream<'static, Connection> {
        self.workspace.changes()
    }
}

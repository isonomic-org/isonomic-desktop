// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The workspace over the shell's client: `models.catalogue`,
//! `models.download` and `models.remove`; and `connections.*`, with its
//! changes from the status subscription.
//!
//! A call waits for the engine to be connected: it may still be starting.

use futures::stream::BoxStream;
use futures::StreamExt as _;
use isonomic_client::{Client, ClientError};
use isonomic_proto::connections::{AddConnection, Connection};
use isonomic_proto::models::{CatalogueEntry, DownloadProgress, ModelDetail, ModelState};
use isonomic_proto::ErrorCode;

use super::{Answer, Catalog, Memory, Model, Workspace};

#[derive(Clone)]
pub struct EngineWorkspace {
    client: Client,
}

impl EngineWorkspace {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

/// An error in the words the person reads. The engine's own are shown as
/// written.
pub fn said(error: ClientError) -> String {
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

pub async fn connected(client: &Client) {
    let mut live = client.live();
    let _ = live.wait_for(|live| live.connected).await;
}

/// The tab a model is under, from what it can do.
fn kind(detail: &ModelDetail) -> &'static str {
    let can = |what: &str| detail.info.supports(what);
    if can("vision") || can("audio") || can("image") {
        "Multimodal"
    } else if can("reasoning") {
        "Reasoning"
    } else if can("chat") {
        "Chat"
    } else if can("embeddings") || can("embed") {
        "Embeddings"
    } else {
        "Utility"
    }
}

/// A model's name without its quantisation tag, as Chat names it.
fn conversation_name(name: &str) -> String {
    name.split(" (").next().unwrap_or(name).trim().to_owned()
}

fn row(entry: CatalogueEntry) -> Model {
    let detail = entry.detail;
    let state = detail.state;
    Model {
        kind: kind(&detail).to_owned(),
        name: conversation_name(&detail.info.name),
        publisher: entry.publisher,
        about: entry
            .about
            .or_else(|| detail.license.clone())
            .unwrap_or_default(),
        size_bytes: detail.size_bytes,
        first: detail.info.recommended,
        installed: !matches!(state, ModelState::NotInstalled | ModelState::Downloading),
        downloading: state == ModelState::Downloading,
        fits: entry.fits,
        fit_reason: entry.fit_reason,
        id: detail.info.id,
    }
}

impl Workspace for EngineWorkspace {
    fn catalogue(&self) -> Answer<Catalog> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            let models = client.models();
            // Below the engine's floor no local model can run: the
            // recommendation says so, with the memory.
            let memory = match models.recommended(None).await {
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
            let entries = models.catalogue().await.map_err(said)?;
            Ok(Catalog {
                models: entries.into_iter().map(row).collect(),
                memory,
            })
        })
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        crate::downloads::download(&self.client, model)
    }

    fn remove_model(&self, model: &str) -> Answer<()> {
        let (client, model) = (self.client.clone(), model.to_owned());
        Box::pin(async move {
            connected(&client).await;
            client.models().remove(&model).await.map_err(said)
        })
    }

    fn connections(&self) -> Answer<Vec<Connection>> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            client.connections().list().await.map_err(said)
        })
    }

    fn connect(&self, add: AddConnection) -> Answer<Connection> {
        let client = self.client.clone();
        Box::pin(async move {
            connected(&client).await;
            client.connections().add(add).await.map_err(said)
        })
    }

    fn sign_in(&self, id: &str) -> Answer<String> {
        let (client, id) = (self.client.clone(), id.to_owned());
        Box::pin(async move {
            connected(&client).await;
            let started = client.connections().sign_in(&id).await.map_err(said)?;
            Ok(started.url)
        })
    }

    fn disconnect(&self, id: &str) -> Answer<()> {
        let (client, id) = (self.client.clone(), id.to_owned());
        Box::pin(async move {
            connected(&client).await;
            client.connections().remove(&id).await.map_err(said)
        })
    }

    fn changes(&self) -> BoxStream<'static, Connection> {
        let mut states = self.client.connections().changes();
        let seen = states.borrow_and_update().clone();
        futures::stream::unfold((states, seen), |(mut states, mut seen)| async move {
            states.changed().await.ok()?;
            let now = states.borrow_and_update().clone();
            let changed: Vec<Connection> = now
                .values()
                .filter(|connection| seen.get(&connection.id) != Some(*connection))
                .cloned()
                .collect();
            seen = now;
            Some((futures::stream::iter(changed), (states, seen)))
        })
        .flatten()
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_loses_its_quantisation_tag() {
        assert_eq!(conversation_name("Chat Small (Q4_K_M)"), "Chat Small");
        assert_eq!(conversation_name("Chat Small"), "Chat Small");
    }
}

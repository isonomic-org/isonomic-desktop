// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The workspace over a real client and the SDK's test engine: the
//! catalogue with its pills and their reasons, downloads and removal; and
//! connections added, signed in with a pasted token or in the browser,
//! followed as they change, and removed.

use std::time::Duration;

use futures::StreamExt as _;
use isonomic_proto::connections::ConnectionState;
use isonomic_testkit::TestEngine;
use isonomic_testkit::SIGN_IN_PAGE;
use shell::connectors;
use shell::workspace::engine::EngineWorkspace;
use shell::workspace::{Memory, Workspace};

fn engine() -> (tokio::runtime::Runtime, TestEngine, EngineWorkspace) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let client = engine.client_on(runtime.handle());
    (runtime, engine, EngineWorkspace::new(client))
}

#[test]
fn the_catalogue_says_which_models_fit_and_why() {
    let (runtime, engine, workspace) = engine();
    engine
        .models()
        .describe("chat-small", Some("A small chat model."), Some("Qwen"));
    engine
        .models()
        .refuse("embed-small", 32 << 30, 16 << 30, "too big");
    let catalog = runtime.block_on(workspace.catalogue()).unwrap();
    assert_eq!(catalog.memory, Memory::Enough);
    let small = catalog
        .models
        .iter()
        .find(|m| m.id == "chat-small")
        .unwrap();
    assert_eq!(small.pill(), "Recommended");
    assert_eq!(small.publisher.as_deref(), Some("Qwen"));
    assert!(small.line().starts_with("Qwen \u{b7} A small chat model."));
    assert_eq!(
        small.reason(),
        "It fits in this computer's 16 GB of memory."
    );
    let embed = catalog
        .models
        .iter()
        .find(|m| m.id == "embed-small")
        .unwrap();
    assert_eq!(embed.pill(), "Not recommended");
    assert_eq!(
        embed.reason(),
        "This computer's 16 GB of memory can't run it."
    );
    // The tabs are the kinds the catalogue has.
    assert_eq!(catalog.tabs(), ["All models", "Chat", "Embeddings"]);
}

#[test]
fn several_models_download_and_one_is_removed() {
    let (runtime, engine, workspace) = engine();
    engine.models().uninstall("chat-small");
    engine.models().uninstall("embed-small");
    runtime.block_on(async {
        for id in ["chat-small", "embed-small"] {
            let events: Vec<_> = workspace.download(id).collect().await;
            assert!(events.last().unwrap().as_ref().unwrap().outcome.is_some());
        }
    });
    assert!(engine.models().installed("chat-small") && engine.models().installed("embed-small"));
    runtime
        .block_on(workspace.remove_model("chat-small"))
        .unwrap();
    assert!(!engine.models().installed("chat-small"));
    // A refusal is the engine's own words.
    let refused = runtime.block_on(workspace.remove_model("chat-small"));
    assert_eq!(refused, Err("chat-small is not installed".to_owned()));
}

#[test]
fn a_token_connector_is_ready_with_its_secret() {
    let (runtime, engine, workspace) = engine();
    let add = connectors::find("github")
        .unwrap()
        .add("token-7f3a")
        .unwrap();
    let connection = runtime.block_on(workspace.connect(add)).unwrap();
    assert_eq!(connection.state, ConnectionState::Ready);
    assert_eq!(connection.key.as_deref(), Some("github"));
    let secret = engine.connections().secret(&connection.id);
    assert_eq!(secret.as_deref(), Some("token-7f3a"));
}

#[test]
fn a_browser_connector_signs_in_and_its_change_arrives() {
    let (runtime, engine, workspace) = engine();
    let client_ready = async {
        // Changes come on the status subscription, once it is open.
        let mut changes = workspace.changes();
        let add = connectors::find("linear").unwrap().add("").unwrap();
        let connection = workspace.connect(add).await.unwrap();
        assert_eq!(connection.state, ConnectionState::NeedsSignIn);
        let page = workspace.sign_in(&connection.id).await.unwrap();
        assert_eq!(page, format!("{SIGN_IN_PAGE}{}", connection.id));
        let changed = tokio::time::timeout(Duration::from_secs(5), changes.next())
            .await
            .expect("the change arrives")
            .unwrap();
        assert_eq!(changed.id, connection.id);
        assert_eq!(changed.state, ConnectionState::Ready);
        connection.id
    };
    let id = runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        client_ready.await
    });
    let failed = runtime.block_on(async {
        let mut changes = workspace.changes();
        engine
            .connections()
            .set_state(&id, ConnectionState::Failed, Some("The server went away."));
        tokio::time::timeout(Duration::from_secs(5), changes.next())
            .await
            .unwrap()
            .unwrap()
    });
    assert_eq!(failed.error.as_deref(), Some("The server went away."));
    runtime.block_on(workspace.disconnect(&id)).unwrap();
    assert!(runtime
        .block_on(workspace.connections())
        .unwrap()
        .is_empty());
    assert!(runtime.block_on(workspace.disconnect(&id)).is_err());
}

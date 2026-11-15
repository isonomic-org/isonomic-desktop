//! The window's engine link against the real engine binary, not the
//! testkit: a scratch Capsule, the engine started by the link because
//! nothing is listening, the database healthy, first run's calls answered,
//! an app granted, and Quit stopping the engine.
//!
//! Skipped unless `ISONOMIC_E2E_ENGINE` names the engine binary, with
//! `ISONOMIC_E2E_BUNDLE` naming the directory of its bundled services.
//! `cargo xtask e2e` sets both. The database refuses to run as root.

use std::path::PathBuf;
use std::time::Duration;

use engine_link::{EngineLink, LinkConfig};
use isonomic_client::Live;
use isonomic_proto::{Capabilities, Manifest, ServiceState};

/// Generous: the first start creates the database.
const START: Duration = Duration::from_secs(90);

fn engine() -> Option<(PathBuf, PathBuf)> {
    let engine = std::env::var_os("ISONOMIC_E2E_ENGINE")?;
    let bundle = std::env::var_os("ISONOMIC_E2E_BUNDLE")?;
    Some((engine.into(), bundle.into()))
}

async fn wait(link: &EngineLink, what: &str, done: impl FnMut(&Live) -> bool) {
    let mut live = link.client().live();
    tokio::time::timeout(START, live.wait_for(done))
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {what}"))
        .expect("the client stopped");
}

#[test]
fn the_window_drives_the_real_engine() {
    let Some((engine, bundle)) = engine() else {
        eprintln!("skipped: ISONOMIC_E2E_ENGINE and ISONOMIC_E2E_BUNDLE are not set");
        return;
    };
    let scratch = tempfile::tempdir().unwrap();
    let capsule = scratch.path().join("Capsule");
    // The engine the link starts inherits these.
    std::env::set_var("ISONOMIC_CAPSULE_DIR", &capsule);
    std::env::set_var("ISONOMIC_BUNDLE_DIR", &bundle);

    let config = LinkConfig::resolve(
        Some(capsule.clone().into()),
        None,
        Some(engine),
        None,
        unsafe_uid(),
    )
    .unwrap();
    let first = config.capsule().prepare().unwrap();
    assert!(first.created && !first.set_up, "a fresh Capsule");
    let link = EngineLink::start(config, None).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        // Nothing listens: the link starts the engine, then connects.
        wait(&link, "the engine to answer", |live| live.connected).await;
        let live = link.client().live().borrow().clone();
        assert!(live.engine.is_some(), "welcome names the engine");
        assert!(live.schema.is_some(), "welcome names the session schema");

        // The database comes up healthy on its own.
        wait(&link, "the database", |live| {
            live.system.as_ref().is_some_and(|system| {
                system
                    .services
                    .iter()
                    .any(|s| s.name == "postgres" && s.state == ServiceState::Healthy)
            })
        })
        .await;

        // First run's calls: the profile, and the recommended model, which
        // the engine answers either way (no model server is bundled here).
        let client = link.client();
        let profile = client.profile().await.unwrap();
        assert!(profile.total_memory_bytes > 0);
        let recommended = client.models().recommended(None).await;
        eprintln!("models.recommended: {recommended:?}");

        // An app is granted from its manifest and calls under its token.
        let manifest = Manifest {
            id: "com.isonomic.e2e".into(),
            name: "End to end".into(),
            version: "0.1.0".into(),
            icon: "dot".into(),
            capabilities: Capabilities::default(),
            schemas: vec![],
        };
        let token = client.grant(manifest).await.unwrap();
        let app = client.scoped(token);
        let denied = app.models().list().await;
        assert!(denied.is_err(), "no models capability: refused");

        // Quit: the engine stops in order and the socket goes away.
        client.shutdown().await.unwrap();
        wait(&link, "the engine to stop", |live| !live.connected).await;
    });
    drop(link);
    runtime.shutdown_timeout(Duration::from_secs(5));
}

#[cfg(unix)]
fn unsafe_uid() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

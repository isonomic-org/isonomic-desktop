//! The link against the in-memory engine: it starts an engine that is not
//! running, does not start it twice, and finds the socket where the engine
//! binds it.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use engine_link::problems::Problems;
use engine_link::{launching_connector, EngineLink, LinkConfig, LinkError};
use isonomic_client::Options;
use isonomic_testkit::TestEngine;

fn fast() -> Options {
    let mut options = Options::new("test");
    options.reconnect_min = Duration::from_millis(5);
    options.reconnect_max = Duration::from_millis(10);
    options
}

async fn connected(link: &EngineLink) {
    let mut live = link.client().live();
    tokio::time::timeout(
        Duration::from_secs(5),
        live.wait_for(|view| view.connected && view.system.is_some()),
    )
    .await
    .expect("never connected")
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn it_starts_an_engine_that_is_not_running_and_only_once() {
    let engine = TestEngine::new();
    engine.go_offline();
    let launches = Arc::new(AtomicU32::new(0));
    let launcher = {
        let (engine, launches) = (engine.clone(), Arc::clone(&launches));
        move || {
            launches.fetch_add(1, Ordering::SeqCst);
            engine.go_online();
            Ok(())
        }
    };
    let connector = launching_connector(
        engine.connector(),
        Arc::new(launcher),
        Duration::from_secs(3600),
        Problems::default(),
    );
    let link = EngineLink::with_connector(connector, fast()).unwrap();
    connected(&link).await;
    assert_eq!(launches.load(Ordering::SeqCst), 1);

    // The engine dies and comes back by itself: the link reconnects without
    // launching a second one inside the relaunch window.
    engine.go_offline();
    let mut live = link.client().live();
    live.wait_for(|view| !view.connected).await.unwrap();
    engine.go_online();
    connected(&link).await;
    assert_eq!(launches.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_running_engine_is_never_launched() {
    let engine = TestEngine::new();
    let launches = Arc::new(AtomicU32::new(0));
    let counter = Arc::clone(&launches);
    let launcher = move || {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    };
    let connector = launching_connector(
        engine.connector(),
        Arc::new(launcher),
        Duration::ZERO,
        Problems::default(),
    );
    let link = EngineLink::with_connector(connector, fast()).unwrap();
    connected(&link).await;
    assert_eq!(launches.load(Ordering::SeqCst), 0);
}

#[test]
fn the_socket_and_the_engine_binary_come_from_plain_inputs() {
    let config = LinkConfig::resolve(
        Some("/data/capsule".into()),
        Some(PathBuf::from("/home/u")),
        None,
        Some(PathBuf::from(
            "/Applications/Isonomic.app/Contents/MacOS/isonomic-desktop",
        )),
        501,
    )
    .unwrap();
    assert_eq!(
        config.socket,
        PathBuf::from("/data/capsule/run/engine.sock")
    );
    assert_eq!(
        config.engine,
        Some(PathBuf::from(
            "/Applications/Isonomic.app/Contents/MacOS/isonomic-capsule"
        ))
    );

    let overridden = LinkConfig::resolve(
        Some("/c".into()),
        None,
        Some(PathBuf::from("/opt/engine")),
        None,
        501,
    )
    .unwrap();
    assert_eq!(overridden.engine, Some(PathBuf::from("/opt/engine")));
    assert!(matches!(
        LinkConfig::resolve(None, None, None, None, 501),
        Err(LinkError::NoCapsuleDir)
    ));
}

#[test]
fn an_engine_that_is_missing_or_dies_on_start_says_why() {
    use engine_link::{EngineBinary, Launcher as _};
    let dir = tempfile::tempdir().unwrap();
    let problems = Problems::default();
    let missing = EngineBinary {
        path: dir.path().join("no-such-engine"),
        log: None,
        problems: problems.clone(),
    };
    assert!(missing.launch().is_err());
    let said = problems.current().unwrap();
    assert!(said.contains("isn't installed"), "{said}");

    // An engine that writes its reason and exits 3.
    let script = dir.path().join("engine");
    std::fs::write(
        &script,
        "#!/bin/sh\necho 'fatal: the database would not start' >&2\nexit 3\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let log = dir.path().join("logs/engine.log");
    let dies = EngineBinary {
        path: script,
        log: Some(log.clone()),
        problems: problems.clone(),
    };
    let mut watch = problems.watch();
    watch.mark_unchanged();
    dies.launch().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(10), watch.changed())
            .await
            .expect("the exit is reported")
            .unwrap();
    });
    let said = problems.current().unwrap();
    assert!(said.contains("exit code 3"), "{said}");
    assert!(said.contains("the database would not start"), "{said}");
    assert!(said.contains("engine.log"), "{said}");

    problems.clear();
    assert_eq!(problems.current(), None);
}

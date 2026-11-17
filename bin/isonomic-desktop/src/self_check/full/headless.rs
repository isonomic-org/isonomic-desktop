// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The full pass's Chat steps (4 to 7), headless: real Chat in the real
//! shell, against the SDK's test engine, which answers every prompt "OK."
//! and forks sessions as the engine does. The steps are the same functions
//! the Mac runs; only the engine and the renderer stand in.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use isonomic_app::{register, MemorySettings};
use isonomic_client::Live;
use isonomic_testkit::TestEngine;
use shell::apps::{AppHost, AppState};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::{mpsc, watch};
use ui::test::TestWindowExt as _;
use ui::{AppContext as _, Entity, TestAppContext, WindowOptions};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// Each step's number and outcome, filled in when the run ends.
type Results = Arc<std::sync::Mutex<Option<Vec<(u8, Result<String, String>)>>>>;

/// How long the whole run may take before the test fails.
const WAIT: Duration = Duration::from_secs(60);

/// Run GPUI and the engine until `done` holds, waking on any of `watched`'s
/// notifications; fail after [`WAIT`].
pub(super) fn run_until<T: 'static>(
    cx: &mut TestAppContext,
    watched: &[Entity<T>],
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let (wake, mut woken) = mpsc::unbounded_channel::<bool>();
    let _observers: Vec<_> = watched
        .iter()
        .map(|entity| {
            let wake = wake.clone();
            cx.update(|app| {
                app.observe(entity, move |_, _| {
                    let _ = wake.send(true);
                })
            })
        })
        .collect();
    let ticker = wake.clone();
    std::thread::spawn(move || {
        // A slow tick, so work finished on the engine's thread is seen.
        let started = Instant::now();
        while started.elapsed() < WAIT {
            std::thread::sleep(Duration::from_millis(20));
            if ticker.send(true).is_err() {
                return;
            }
        }
        let _ = ticker.send(false);
    });
    cx.executor().allow_parking();
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        let executor = cx.foreground_executor().clone();
        match executor.block_test(woken.recv()) {
            Some(true) => continue,
            _ => panic!("the full pass did not finish within {WAIT:?}"),
        }
    }
}

/// Real Chat in the real shell, on the SDK's test engine, with its tab open.
pub(super) fn mount(
    cx: &mut TestAppContext,
) -> (
    tokio::runtime::Runtime,
    TestEngine,
    ui::AnyWindowHandle,
    Entity<Shell>,
    Entity<isonomic_chat::ChatShell>,
) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let host = AppHost::new(
        engine.client_on(runtime.handle()),
        Arc::new(MemorySettings::default()),
    );
    let (_, live) = watch::channel(Live::default());
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        ui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.mount(register::<isonomic_chat::Chat>(), host, cx);
                shell
            })
        })
        .expect("open test window")
    });
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("rail-com.isonomic.chat", cx);
    })
    .unwrap();
    let tab = shell
        .read_with(cx, |s, _| s.app("com.isonomic.chat").cloned())
        .unwrap();
    run_until(cx, std::slice::from_ref(&tab), |cx| {
        tab.read_with(cx, |t, _| matches!(t.state(), AppState::Open(_)))
    });
    let chat = tab.read_with(cx, |t, cx| match t.state() {
        AppState::Open(app) => app
            .view()
            .downcast::<isonomic_chat::Chat>()
            .ok()
            .map(|c| c.read(cx).shell().clone()),
        _ => None,
    });
    let chat = chat.expect("Chat's root view");
    (runtime, engine, window, shell, chat)
}

#[ui::test]
fn chat_steps_pass_against_the_test_engine(cx: &mut TestAppContext) {
    let (_runtime, _engine, window, shell, chat) = mount(cx);

    // The prompt run's first chat: two finished turns, as after continue.
    let started = Instant::now();
    let results: Results = Arc::default();
    let into = Arc::clone(&results);
    let (chat_for, window_for, shell_for) = (chat.clone(), window, shell.clone());
    cx.spawn(move |mut cx| async move {
        let cx = &mut cx;
        let mut out = Vec::new();
        let first = super::new_chat(window_for, &chat_for, cx).await;
        let Ok(first) = first else {
            out.push((0, first.map(|_| String::new())));
            *into.lock().unwrap() = Some(out);
            return;
        };
        let view = cx
            .update(|cx| chat_for.read(cx).thread(&first).cloned())
            .expect("a thread");
        cx.update(|cx| view.update(cx, |v, cx| v.send("And tomorrow?".into(), cx)));
        let _ = super::super::until_within(cx, &view, "turn two", super::MODEL_STEP, |v, _| {
            v.thread().turns().len() == 2 && super::idle(v)
        })
        .await;
        out.push((4, super::try_again(window_for, &view, started, cx).await));
        out.push((5, super::edit(window_for, &view, cx).await));
        out.push((
            6,
            super::chat_list(window_for, &chat_for, started, cx).await,
        ));
        out.push((7, super::search(window_for, &chat_for, started, cx).await));
        out.push((
            10,
            super::more::picker(window_for, &chat_for, started, cx).await,
        ));
        let capsule = tempfile::tempdir().expect("a scratch folder");
        out.push((
            12,
            super::more::notice_screen(window_for, &shell_for, capsule.path(), started, cx).await,
        ));
        // What a relaunch reads: the list from Chat's store, as in memory.
        let kept = cx.update(|cx| chat_for.read(cx).sessions().len());
        let before = cx.update(|cx| chat_for.read(cx).loads_finished());
        cx.update(|cx| chat_for.update(cx, |s, cx| s.reload(cx)));
        let reread = super::super::until_within(
            cx,
            &chat_for,
            "the list read again",
            super::UI_STEP,
            |s, _| s.loads_finished() > before,
        )
        .await
        .map(|_| cx.update(|cx| chat_for.read(cx).sessions().len()));
        out.push((
            9,
            match reread {
                Ok(n) if n == kept && n == 2 => Ok(format!("{n} chats in memory and in the store")),
                Ok(n) => Err(format!("{kept} chats in memory, {n} in the store")),
                Err(e) => Err(e),
            },
        ));
        *into.lock().unwrap() = Some(out);
    })
    .detach();
    run_until(cx, std::slice::from_ref(&chat), |_| {
        results.lock().unwrap().is_some()
    });
    let results = results.lock().unwrap().take().unwrap();
    for (n, outcome) in &results {
        eprintln!("step {n}: {outcome:?}");
    }
    let failed: Vec<_> = results.iter().filter(|(_, r)| r.is_err()).collect();
    assert!(failed.is_empty(), "{failed:?}");
    assert_eq!(results.len(), 7);
}

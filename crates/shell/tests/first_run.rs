//! First run in a headless window, in the real shell. The hardware profile
//! and the marker are scripted: the SDK testkit's profile is fixed at the
//! floor, so a smaller machine can't be served by it yet (isonomic-sdk #21).

extern crate ui as gpui;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use isonomic_client::Live;
use isonomic_proto::HardwareProfile;
use isonomic_testkit::TestEngine;
use shell::first_run::{Memory, Setup, MEMORY_FLOOR_BYTES};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

/// Answers each profile request with the next scripted answer.
struct Scripted {
    answers: Mutex<Vec<Result<u64, String>>>,
    finished: AtomicUsize,
}

impl Scripted {
    fn new(answers: Vec<Result<u64, String>>) -> Arc<Self> {
        Arc::new(Self {
            answers: Mutex::new(answers),
            finished: AtomicUsize::new(0),
        })
    }
}

impl Setup for Scripted {
    fn profile(&self) -> BoxFuture<'static, Result<HardwareProfile, String>> {
        let answer = self.answers.lock().unwrap().remove(0);
        Box::pin(async move {
            answer.map(|total| HardwareProfile {
                profile: "standard".into(),
                total_memory_bytes: total,
                available_memory_bytes: total / 2,
            })
        })
    }

    fn finish(&self) -> BoxFuture<'static, Result<(), String>> {
        self.finished.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

fn open(cx: &mut TestAppContext, setup: Arc<dyn Setup>) -> (AnyWindowHandle, Entity<Shell>) {
    let (_, live) = watch::channel(Live::default());
    cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(900.), px(600.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.start_first_run(setup, true, cx);
                shell
            })
        })
        .expect("open test window")
    })
}

fn shown(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id).map(|e| e.visible()).unwrap_or(false)
    })
    .unwrap()
}

fn click(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click(id, cx);
    })
    .unwrap();
    cx.run_until_parked();
}

#[ui::test]
fn first_run_checks_memory_then_shows_the_tabs(cx: &mut TestAppContext) {
    let setup = Scripted::new(vec![Ok(MEMORY_FLOOR_BYTES)]);
    let (window, shell) = open(cx, setup.clone());
    cx.run_until_parked();

    assert!(shown(cx, window, "first-run"));
    assert!(shown(cx, window, "capsule-ready"));
    assert!(shown(cx, window, "memory-enough"));
    assert!(shown(cx, window, "model-download-unavailable"));
    assert!(!shown(cx, window, "rail"), "the tabs wait for first run");

    click(cx, window, "first-run-continue");
    assert_eq!(setup.finished.load(Ordering::SeqCst), 1);
    assert!(shell.read_with(cx, |shell, _| shell.first_run().is_none()));
    assert!(!shown(cx, window, "first-run"));
    assert!(shown(cx, window, "rail-home"));
}

#[ui::test]
fn below_the_floor_first_run_cannot_finish(cx: &mut TestAppContext) {
    let setup = Scripted::new(vec![Ok(8 << 30)]);
    let (window, shell) = open(cx, setup.clone());
    cx.run_until_parked();

    assert!(shown(cx, window, "memory-too-little"));
    click(cx, window, "first-run-continue");
    assert_eq!(setup.finished.load(Ordering::SeqCst), 0);
    assert!(shell.read_with(cx, |shell, _| shell.first_run().is_some()));
}

#[ui::test]
fn a_failed_check_can_be_tried_again(cx: &mut TestAppContext) {
    let setup = Scripted::new(vec![Err("engine not reachable".into()), Ok(16 << 30)]);
    let (window, shell) = open(cx, setup);
    cx.run_until_parked();

    assert!(shown(cx, window, "memory-failed"));
    click(cx, window, "memory-check-again");
    let first_run = shell
        .read_with(cx, |shell, _| shell.first_run().cloned())
        .unwrap();
    let memory = first_run.read_with(cx, |first_run, _| first_run.memory().clone());
    assert!(matches!(memory, Memory::Enough(_)));
}

/// The real client against the testkit's profile, which sits at the floor.
#[test]
fn the_testkit_machine_meets_the_floor() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let client = engine.client_on(runtime.handle());
    let profile = runtime
        .block_on(shell::first_run::profile_from(&client))
        .unwrap();
    assert!(profile.total_memory_bytes >= MEMORY_FLOOR_BYTES);
}

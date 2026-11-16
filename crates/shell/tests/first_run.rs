//! First run in a headless window, in the real shell. The view's setup is
//! scripted, because GPUI's test scheduler refuses wakeups from the
//! client's runtime thread. The adapter over the real client is tested
//! against the testkit at the end, outside GPUI.

extern crate ui as gpui;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use futures::channel::mpsc;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_client::Live;
use isonomic_proto::models::{DownloadOutcome, DownloadProgress, Recommendation};
use isonomic_proto::HardwareProfile;
use isonomic_testkit::TestEngine;
use shell::first_run::{Memory, ModelStep, Setup, MEMORY_FLOOR_BYTES};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext, WindowBounds,
    WindowOptions,
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
    installed: bool,
    /// Each download plays what the test sends on its channel.
    downloads: Mutex<Vec<mpsc::UnboundedReceiver<Result<DownloadProgress, String>>>>,
    cancelled: AtomicUsize,
    /// Each ID request takes the next answer; the last one repeats.
    identities: Mutex<Vec<Result<String, String>>>,
}

const MODEL: &str = "chat-small";
const SIZE: u64 = 2 << 30;

fn progress(received: u64, outcome: Option<DownloadOutcome>) -> Result<DownloadProgress, String> {
    Ok(DownloadProgress {
        model: MODEL.into(),
        received_bytes: received,
        total_bytes: Some(SIZE),
        outcome,
        error: None,
    })
}

impl Scripted {
    fn new(answers: Vec<Result<u64, String>>) -> Arc<Self> {
        Self::with(answers, true, vec![])
    }

    fn with(
        answers: Vec<Result<u64, String>>,
        installed: bool,
        downloads: Vec<mpsc::UnboundedReceiver<Result<DownloadProgress, String>>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            answers: Mutex::new(answers),
            finished: AtomicUsize::new(0),
            installed,
            downloads: Mutex::new(downloads),
            cancelled: AtomicUsize::new(0),
            identities: Mutex::new(vec![Ok(ID.into())]),
        })
    }
}

const ID: &str = "iso1-7kq2-m9xd-4tfa";

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

    fn recommended(&self) -> BoxFuture<'static, Result<Recommendation, String>> {
        Box::pin(async {
            Ok(Recommendation {
                model: MODEL.into(),
                profile: "standard".into(),
                size_bytes: Some(SIZE),
            })
        })
    }

    fn installed(&self, _: &str) -> BoxFuture<'static, Result<bool, String>> {
        let installed = self.installed;
        Box::pin(async move { Ok(installed) })
    }

    fn download(&self, _: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        Box::pin(self.downloads.lock().unwrap().remove(0))
    }

    fn cancel_download(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        self.cancelled.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }

    fn identity(&self) -> BoxFuture<'static, Result<String, String>> {
        let mut answers = self.identities.lock().unwrap();
        let answer = if answers.len() > 1 {
            answers.remove(0)
        } else {
            answers[0].clone()
        };
        Box::pin(async move { answer })
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
                size: ui::tokens::window_size(),
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
    assert!(
        shown(cx, window, "model-installed"),
        "already installed: no download"
    );
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

fn step(cx: &mut TestAppContext, shell: &Entity<Shell>) -> ModelStep {
    let first_run = shell
        .read_with(cx, |shell, _| shell.first_run().cloned())
        .unwrap();
    first_run.read_with(cx, |first_run, _| first_run.model().clone())
}

#[ui::test]
fn the_model_downloads_with_progress_then_first_run_can_finish(cx: &mut TestAppContext) {
    let (first, first_events) = mpsc::unbounded();
    let (second, second_events) = mpsc::unbounded();
    let (third, third_events) = mpsc::unbounded();
    let setup = Scripted::with(
        vec![Ok(16 << 30)],
        false,
        vec![first_events, second_events, third_events],
    );
    let (window, shell) = open(cx, setup.clone());
    cx.run_until_parked();

    // Not installed: the person starts it, and Continue waits.
    assert!(shown(cx, window, "model-ready"));
    click(cx, window, "first-run-continue");
    assert_eq!(setup.finished.load(Ordering::SeqCst), 0);

    // Progress shows as it arrives; cancelling asks the engine, and its
    // last event puts the step back.
    click(cx, window, "model-download");
    first.unbounded_send(progress(SIZE / 4, None)).unwrap();
    cx.run_until_parked();
    assert!(matches!(
        step(cx, &shell),
        ModelStep::Downloading { received, total, .. } if received == SIZE / 4 && total == Some(SIZE)
    ));
    assert!(shown(cx, window, "model-downloading"));
    click(cx, window, "model-cancel");
    assert_eq!(setup.cancelled.load(Ordering::SeqCst), 1);
    first
        .unbounded_send(progress(SIZE / 4, Some(DownloadOutcome::Cancelled)))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, "model-ready"));

    // The next attempt fails with the engine's reason and can be retried.
    click(cx, window, "model-download");
    second
        .unbounded_send(Ok(DownloadProgress {
            outcome: Some(DownloadOutcome::Failed),
            error: Some(isonomic_proto::WireError::new(
                isonomic_proto::ErrorCode::ModelUnavailable,
                "the network went away",
            )),
            ..progress(SIZE / 2, None).unwrap()
        }))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, "model-failed"));
    assert!(matches!(
        step(cx, &shell),
        ModelStep::Failed { error, .. } if error == "the network went away"
    ));

    // Retry: installed, and first run can finish.
    click(cx, window, "model-retry");
    third.unbounded_send(progress(SIZE, None)).unwrap();
    third
        .unbounded_send(progress(SIZE, Some(DownloadOutcome::Installed)))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, "model-installed"));
    click(cx, window, "first-run-continue");
    assert_eq!(setup.finished.load(Ordering::SeqCst), 1);
    assert!(shown(cx, window, "rail-home"));
}

#[ui::test]
fn a_stream_that_ends_without_an_outcome_is_a_failure(cx: &mut TestAppContext) {
    let (first, first_events) = mpsc::unbounded();
    let setup = Scripted::with(vec![Ok(16 << 30)], false, vec![first_events]);
    let (window, _shell) = open(cx, setup);
    cx.run_until_parked();
    click(cx, window, "model-download");
    first.unbounded_send(progress(1, None)).unwrap();
    drop(first);
    cx.run_until_parked();
    assert!(shown(cx, window, "model-failed"));
}

/// The adapter over the real client, against the testkit: the recommended
/// model downloads to installed, with progress on the way.
#[test]
fn the_client_adapter_downloads_the_recommended_model() {
    use futures::StreamExt as _;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let client = engine.client_on(runtime.handle());
    runtime.block_on(async {
        let model = shell::downloads::recommended(&client).await.unwrap();
        engine.models().uninstall(&model.model);
        assert!(!shell::downloads::installed(&client, &model.model)
            .await
            .unwrap());
        let events: Vec<_> = shell::downloads::download(&client, &model.model)
            .collect()
            .await;
        let last = events.last().unwrap().as_ref().unwrap();
        assert_eq!(last.outcome, Some(DownloadOutcome::Installed));
        assert!(shell::downloads::installed(&client, &model.model)
            .await
            .unwrap());
    });
}

#[ui::test]
fn an_unknown_size_shows_bytes_so_far_and_never_divides(cx: &mut TestAppContext) {
    let (first, first_events) = mpsc::unbounded();
    let setup = Scripted::with(vec![Ok(16 << 30)], false, vec![first_events]);
    let (window, shell) = open(cx, setup);
    cx.run_until_parked();
    // The recommendation knows the size; the engine's progress doesn't,
    // and neither does a model whose recommendation has none.
    click(cx, window, "model-download");
    first
        .unbounded_send(Ok(DownloadProgress {
            total_bytes: None,
            ..progress(3 << 20, None).unwrap()
        }))
        .unwrap();
    cx.run_until_parked();
    assert!(matches!(
        step(cx, &shell),
        ModelStep::Downloading {
            total: Some(SIZE),
            ..
        }
    ));
    assert!(shown(cx, window, "model-downloading"));
}

#[ui::test]
fn first_run_can_be_finished_by_keyboard(cx: &mut TestAppContext) {
    let setup = Scripted::new(vec![Ok(MEMORY_FLOOR_BYTES)]);
    let (window, shell) = open(cx, setup.clone());
    cx.run_until_parked();
    let mut visual = ui::VisualTestContext::from_window(window, cx);
    let focus = shell.clone();
    visual.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
        focus.update(cx, |s, cx| s.focus_rail(window, cx));
        window.draw(cx).clear(cx);
    });
    // Copy the ID, then Continue: the two buttons once the model is installed.
    for key in ["tab", "tab", "enter"] {
        let keystroke = ui::Keystroke::parse(key).unwrap();
        visual.simulate_event(ui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        visual.simulate_event(ui::KeyUpEvent { keystroke });
        visual.update(|window, cx| window.draw(cx).clear(cx));
    }
    visual.run_until_parked();
    assert_eq!(setup.finished.load(Ordering::SeqCst), 1);
    assert!(shell.read_with(&visual, |s, _| s.first_run().is_none()));
}

#[ui::test]
fn the_isonomic_id_is_shown_to_read_and_copy(cx: &mut TestAppContext) {
    let setup = Scripted::new(vec![Ok(MEMORY_FLOOR_BYTES)]);
    let (window, shell) = open(cx, setup);
    cx.run_until_parked();
    let first_run = shell.read_with(cx, |s, _| s.first_run().cloned()).unwrap();
    assert_eq!(
        first_run.read_with(cx, |f, _| f.identity().clone()),
        shell::first_run::IdStep::Ready(ID.into())
    );
    assert!(shown(cx, window, "id-ready"));
    assert!(shown(cx, window, "id-copy"));
}

#[ui::test]
fn a_refused_key_store_says_why_and_can_be_tried_again(cx: &mut TestAppContext) {
    let refused = "The Mac's keychain is locked. Unlock it and try again.";
    let setup = Scripted::new(vec![Ok(MEMORY_FLOOR_BYTES)]);
    *setup.identities.lock().unwrap() = vec![Err(refused.into()), Ok(ID.into())];
    let (window, shell) = open(cx, setup.clone());
    cx.run_until_parked();
    let first_run = shell.read_with(cx, |s, _| s.first_run().cloned()).unwrap();
    assert_eq!(
        first_run.read_with(cx, |f, _| f.identity().clone()),
        shell::first_run::IdStep::Failed(refused.into())
    );
    assert!(shown(cx, window, "id-failed"));
    // No ID: the model waits and Continue does nothing.
    assert!(matches!(
        first_run.read_with(cx, |f, _| f.model().clone()),
        ModelStep::Waiting
    ));
    click(cx, window, "first-run-continue");
    assert_eq!(setup.finished.load(Ordering::SeqCst), 0);

    click(cx, window, "id-retry");
    assert!(shown(cx, window, "id-ready"));
    assert!(shown(cx, window, "model-installed"));
}

/// The adapter over the real client, against the testkit's identity: the
/// ID as the engine gives it, and a refusal's words as written.
#[test]
fn the_client_adapter_reads_the_id_and_a_refusal() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let engine = runtime.block_on(async { TestEngine::new() });
    let client = engine.client_on(runtime.handle());
    runtime.block_on(async {
        let id = shell::first_run::identity_from(&client).await.unwrap();
        assert_eq!(id, isonomic_testkit::DEFAULT_IDENTITY);
        let refused =
            "Your Mac's keychain did not let Isonomic save your ID. Unlock it and try again.";
        engine.identity().refuse(refused);
        assert_eq!(
            shell::first_run::identity_from(&client).await,
            Err(refused.to_string())
        );
        engine.identity().recover();
        assert_eq!(shell::first_run::identity_from(&client).await.unwrap(), id);
    });
}

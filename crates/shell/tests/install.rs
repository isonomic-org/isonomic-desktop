//! An app's request to install a model, in the real shell, headless. The
//! source is scripted event by event, as in first run's tests.

extern crate ui as gpui;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use futures::channel::{mpsc, oneshot};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_client::Live;
use isonomic_proto::models::{DownloadOutcome, DownloadProgress};
use shell::install::{InstallOutcome, ModelSource};
use shell::logs::LogSource;
use shell::Shell;
use tokio::sync::watch;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
    WindowBounds, WindowOptions,
};

const MODEL: &str = "chat-medium";

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

struct Scripted {
    installed: Result<bool, String>,
    downloads: Mutex<Vec<mpsc::UnboundedReceiver<Result<DownloadProgress, String>>>>,
    cancelled: AtomicUsize,
}

impl ModelSource for Scripted {
    fn installed(&self, _: &str) -> BoxFuture<'static, Result<bool, String>> {
        let answer = self.installed.clone();
        Box::pin(async move { answer })
    }

    fn download(&self, _: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        Box::pin(self.downloads.lock().unwrap().remove(0))
    }

    fn cancel_download(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        self.cancelled.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}

fn progress(received: u64, outcome: Option<DownloadOutcome>) -> Result<DownloadProgress, String> {
    Ok(DownloadProgress {
        model: MODEL.into(),
        received_bytes: received,
        total_bytes: Some(1000),
        outcome,
        error: None,
    })
}

fn open(
    cx: &mut TestAppContext,
    source: Arc<Scripted>,
) -> (
    AnyWindowHandle,
    Entity<Shell>,
    oneshot::Receiver<InstallOutcome>,
) {
    let (_, live) = watch::channel(Live::default());
    let mut asked = None;
    let (window, shell) = cx.update(|cx| {
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
                asked = Some(shell.install_model(MODEL.into(), "Chat".into(), source, cx));
                shell
            })
        })
        .expect("open test window")
    });
    (window, shell, asked.unwrap())
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

fn source(
    installed: Result<bool, String>,
    downloads: usize,
) -> (
    Arc<Scripted>,
    Vec<mpsc::UnboundedSender<Result<DownloadProgress, String>>>,
) {
    let (senders, receivers): (Vec<_>, Vec<_>) = (0..downloads).map(|_| mpsc::unbounded()).unzip();
    let source = Arc::new(Scripted {
        installed,
        downloads: Mutex::new(receivers),
        cancelled: AtomicUsize::new(0),
    });
    (source, senders)
}

#[ui::test]
fn a_missing_model_downloads_and_the_request_resolves_installed(cx: &mut TestAppContext) {
    let (source, senders) = source(Ok(false), 2);
    let (window, shell, mut asked) = open(cx, source.clone());
    cx.run_until_parked();
    assert!(shown(cx, window, "install-ready"));

    // A failed attempt says why and can be retried; cancelling asks the engine.
    click(cx, window, "install-download");
    senders[0].unbounded_send(progress(400, None)).unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, "install-downloading"));
    click(cx, window, "install-cancel");
    assert_eq!(source.cancelled.load(Ordering::SeqCst), 1);
    senders[0]
        .unbounded_send(Err("the network went away".into()))
        .unwrap();
    cx.run_until_parked();
    assert!(shown(cx, window, "install-failed"));
    assert_eq!(asked.try_recv().unwrap(), None, "not resolved yet");

    click(cx, window, "install-retry");
    senders[1]
        .unbounded_send(progress(1000, Some(DownloadOutcome::Installed)))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(asked.try_recv().unwrap(), Some(InstallOutcome::Installed));
    assert!(shell.read_with(cx, |s, _| s.model_install().is_none()));
    assert!(shown(cx, window, "rail-home"));
}

#[ui::test]
fn not_now_resolves_declined_and_an_installed_model_resolves_at_once(cx: &mut TestAppContext) {
    let (source, _senders) = source(Ok(false), 0);
    let (window, _shell, mut asked) = open(cx, source);
    cx.run_until_parked();
    click(cx, window, "install-not-now");
    assert_eq!(asked.try_recv().unwrap(), Some(InstallOutcome::Declined));
    assert!(!shown(cx, window, "model-install"));
}

#[ui::test]
fn an_installed_model_resolves_without_showing_anything(cx: &mut TestAppContext) {
    let (source, _senders) = source(Ok(true), 0);
    let (window, _shell, mut asked) = open(cx, source);
    cx.run_until_parked();
    assert_eq!(asked.try_recv().unwrap(), Some(InstallOutcome::Installed));
    assert!(!shown(cx, window, "model-install"));
}

/// An app's `AppContext::request(InstallModel)` reaches the window through
/// the SDK's trait and shows the download, named for the app.
#[ui::test]
fn an_apps_request_shows_the_download_named_for_the_app(cx: &mut TestAppContext) {
    use isonomic_app::{WindowRequest, WindowRequests as _};
    let (source, _senders) = source(Ok(false), 0);
    let (requests, queued) = shell::install::requests();
    let (_, live) = watch::channel(Live::default());
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        ui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.serve_requests(queued, source, cx);
                shell
            })
        })
        .expect("open test window")
    });
    let taken = requests.request(
        "com.isonomic.chat",
        WindowRequest::InstallModel {
            model: MODEL.into(),
        },
    );
    assert!(taken);
    cx.run_until_parked();
    let install = shell
        .read_with(cx, |s, _| s.model_install().cloned())
        .unwrap();
    assert_eq!(install.read_with(cx, |i, _| i.model().to_string()), MODEL);
    assert!(shown(cx, window, "install-ready"));
}

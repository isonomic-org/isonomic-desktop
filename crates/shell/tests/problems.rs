//! Problem-report notices in a headless window: what is asked, what Don't
//! send and Keep do, the setting, and the Isonomic ID checkbox.

extern crate ui as gpui;

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::Live;
use reports::kept::Kept;
use reports::settings::{self, Mode, ReportSettings};
use reports::store::Store;
use serde_json::json;
use shell::logs::LogSource;
use shell::problems::ProblemReports;
use shell::Shell;
use tokio::sync::watch;
use ui::component::WindowExt as _;
use ui::test::TestWindowExt as _;
use ui::{AnyWindowHandle, AppContext as _, Entity, TestAppContext, WindowOptions};

struct NoLogs;

impl LogSource for NoLogs {
    fn last_lines(&self, _: &str) -> BoxFuture<'static, Result<Vec<String>, String>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn retry(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

fn keep_report(capsule: &std::path::Path, kind: &str, at: u128) {
    let document = json!({ "format": 1, "kind": kind, "component": { "id": "window" } });
    Store::in_capsule(capsule).save(&document, at).unwrap();
}

fn open(
    cx: &mut TestAppContext,
    capsule: &std::path::Path,
) -> (AnyWindowHandle, Entity<ProblemReports>, Entity<Shell>) {
    let (_, live) = watch::channel(Live::default());
    let kept = Arc::new(Kept::in_capsule(capsule));
    let (window, shell) = cx.update(|cx| {
        ui::init(cx);
        ui::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|cx| {
                let mut shell = Shell::new(live, Arc::new(NoLogs), cx);
                shell.watch_reports(kept, cx);
                shell
            })
        })
        .expect("open test window")
    });
    draw(cx, window);
    let reports = shell.read_with(cx, |s, _| s.reports().cloned()).unwrap();
    (window, reports, shell)
}

fn draw(cx: &mut TestAppContext, window: AnyWindowHandle) {
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn shown(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> bool {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id).is_some_and(|e| e.visible())
    })
    .unwrap()
}

fn click(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click(id, cx);
    })
    .unwrap();
    draw(cx, window);
}

fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, window, cx| window.has_active_dialog(cx))
        .unwrap()
}

fn title(cx: &mut TestAppContext, reports: &Entity<ProblemReports>) -> String {
    reports.read_with(cx, |r, _| r.title())
}

#[ui::test]
fn a_kept_crash_is_asked_about_and_dont_send_deletes_it(cx: &mut TestAppContext) {
    let capsule = tempfile::tempdir().unwrap();
    keep_report(capsule.path(), "panic", 1);
    keep_report(capsule.path(), "hang", 2);
    let (window, reports, _) = open(cx, capsule.path());

    assert!(dialog_open(cx, window));
    assert_eq!(title(cx, &reports), "Isonomic quit unexpectedly");
    assert!(shown(cx, window, "report-notice"));
    // No sender and no ID yet: neither Send nor the checkbox is offered.
    assert!(!reports.read_with(cx, |r, _| r.offers_send()));
    assert!(!shown(cx, window, "report-include-id"));

    // Show report: exactly the kept bytes.
    let file = Kept::in_capsule(capsule.path()).pending().remove(0);
    click(cx, window, "report-show");
    assert!(shown(cx, window, "report-bytes"));
    assert_eq!(reports.read_with(cx, |r, _| r.outgoing()), Some(file.bytes));
    click(cx, window, "report-back");
    assert!(shown(cx, window, "report-notice"));

    // Don't send deletes it, and the next report is asked about.
    click(cx, window, "report-dont-send");
    assert!(!file.path.exists());
    assert!(dialog_open(cx, window));
    assert_eq!(title(cx, &reports), "Isonomic stopped responding");

    // Keep leaves it on this Mac and it isn't asked about again.
    click(cx, window, "report-keep");
    assert!(!dialog_open(cx, window));
    assert_eq!(Kept::in_capsule(capsule.path()).pending().len(), 1);
    cx.update_window(window, |_, window, cx| {
        reports.update(cx, |r, cx| r.review(window, cx))
    })
    .unwrap();
    draw(cx, window);
    assert!(!dialog_open(cx, window));
}

#[ui::test]
fn never_send_deletes_quietly_and_the_choice_is_kept(cx: &mut TestAppContext) {
    let capsule = tempfile::tempdir().unwrap();
    let never = ReportSettings { mode: Mode::Never };
    settings::save(capsule.path(), &never).unwrap();
    keep_report(capsule.path(), "panic", 1);
    let (window, reports, _) = open(cx, capsule.path());
    // The setting is read before the first review.
    cx.update_window(window, |_, window, cx| {
        reports.update(cx, |r, cx| r.review(window, cx))
    })
    .unwrap();
    draw(cx, window);
    assert!(!dialog_open(cx, window));
    assert!(Kept::in_capsule(capsule.path()).pending().is_empty());
    assert!(shown(cx, window, "report-setting"));

    // Send automatically, with nowhere to send: kept, and nothing asked.
    reports.update(cx, |r, cx| r.set_mode(Mode::Always, cx));
    keep_report(capsule.path(), "hang", 2);
    cx.update_window(window, |_, window, cx| {
        reports.update(cx, |r, cx| r.review(window, cx))
    })
    .unwrap();
    draw(cx, window);
    assert!(!dialog_open(cx, window));
    assert_eq!(Kept::in_capsule(capsule.path()).pending().len(), 1);
    assert_eq!(settings::load(capsule.path()).mode, Mode::Always);
}

#[ui::test]
fn the_isonomic_id_is_off_until_ticked(cx: &mut TestAppContext) {
    let capsule = tempfile::tempdir().unwrap();
    keep_report(capsule.path(), "panic", 1);
    let (window, reports, shell) = open(cx, capsule.path());
    shell.update(cx, |shell, cx| {
        shell.report_identity(Box::pin(async { Ok("ISO-TEST".to_string()) }), cx)
    });
    cx.run_until_parked();
    draw(cx, window);
    assert!(shown(cx, window, "report-include-id"));
    assert!(!reports.read_with(cx, |r, _| r.includes_id()));
    let plain = reports.read_with(cx, |r, _| r.outgoing()).unwrap();
    assert!(!plain.contains("ISO-TEST"));

    click(cx, window, "report-include-id");
    let with_id = reports.read_with(cx, |r, _| r.outgoing()).unwrap();
    assert!(
        with_id.contains("\"isonomic_id\": \"ISO-TEST\""),
        "{with_id}"
    );
    // The kept file never holds the ID.
    let file = Kept::in_capsule(capsule.path()).pending().remove(0);
    assert!(!file.bytes.contains("ISO-TEST"));
}

#[ui::test]
fn a_service_failing_while_open_is_asked_about(cx: &mut TestAppContext) {
    use isonomic_proto::{ServiceState, ServiceStatus, SystemStatus};
    let capsule = tempfile::tempdir().unwrap();
    let (publish, live) = watch::channel(Live::default());
    let (window, reports, shell) = open(cx, capsule.path());
    shell.update(cx, |shell, cx| shell.review_on_status(live, cx));
    assert!(!dialog_open(cx, window));
    // The engine writes its report, then the service shows as degraded.
    let failed = json!({ "format": 1, "kind": "service_failed",
        "component": { "id": "engine/postgres" }, "service": { "name": "postgres" } });
    Store::in_capsule(capsule.path()).save(&failed, 1).unwrap();
    let system = SystemStatus {
        aggregate: ServiceState::Degraded,
        services: vec![ServiceStatus {
            name: "postgres".into(),
            tier: 1,
            state: ServiceState::Degraded,
            restart_count: 5,
            last_healthy_unix: None,
        }],
        cold_start_ms: None,
    };
    publish.send_modify(|l| {
        l.connected = true;
        l.system = Some(system);
    });
    draw(cx, window);
    draw(cx, window);
    assert!(dialog_open(cx, window));
    assert_eq!(title(cx, &reports), "The database stopped working");
}

#[ui::test]
fn a_report_written_after_the_status_is_still_asked_about(cx: &mut TestAppContext) {
    let capsule = tempfile::tempdir().unwrap();
    let (publish, live) = watch::channel(Live::default());
    let (window, reports, shell) = open(cx, capsule.path());
    shell.update(cx, |shell, cx| shell.review_on_status(live, cx));
    // The status changes first; the engine's report follows it.
    publish.send_modify(|l| l.connected = true);
    draw(cx, window);
    assert!(!dialog_open(cx, window));
    let failed = json!({ "format": 1, "kind": "model_server_failed",
        "component": { "id": "engine/llama-server" } });
    Store::in_capsule(capsule.path()).save(&failed, 1).unwrap();
    cx.executor()
        .advance_clock(ui::tokens::report_rescan_delay());
    draw(cx, window);
    draw(cx, window);
    assert!(dialog_open(cx, window));
    assert_eq!(title(cx, &reports), "The local model stopped working");
}

#[ui::test]
fn a_problem_an_app_reports_is_written_and_asked_about(cx: &mut TestAppContext) {
    use isonomic_app::{Problem, WindowRequest, WindowRequests as _};
    let capsule = tempfile::tempdir().unwrap();
    let (window, reports, shell) = open(cx, capsule.path());
    let writer = Arc::new(reports::panic::Context {
        store: Store::in_capsule(capsule.path()),
        build: reports::Build::default(),
        system: std::sync::Mutex::new(reports::System::default()),
        home: Some("/Users/omar".into()),
        apps: Vec::new(),
        tab: std::sync::Mutex::new(None),
        started: std::time::Instant::now(),
    });
    reports.update(cx, |r, _| r.write_with(writer));
    // The app's request goes through the window's queue, as from an app.
    let (requests, queued) = shell::install::requests();
    struct NoModels;
    impl shell::install::ModelSource for NoModels {
        fn installed(&self, _: &str) -> BoxFuture<'static, Result<bool, String>> {
            Box::pin(async { Ok(true) })
        }
        fn download(
            &self,
            _: &str,
        ) -> futures::stream::BoxStream<
            'static,
            Result<isonomic_proto::models::DownloadProgress, String>,
        > {
            Box::pin(futures::stream::empty())
        }
        fn cancel_download(&self, _: &str) -> BoxFuture<'static, Result<(), String>> {
            Box::pin(async { Ok(()) })
        }
    }
    let problem = Problem {
        summary: "Couldn't save the chat list".into(),
        detail: Some(
            "at /Users/omar/Library/x, token iso_tok_7f3a9c21b04e5d6f8a1b2c3d4e5f6a7b".into(),
        ),
    };
    assert!(requests.request("com.isonomic.test-notes", WindowRequest::Report(problem)));
    shell.update(cx, |shell, cx| {
        shell.serve_requests(queued, Arc::new(NoModels), cx)
    });
    draw(cx, window);
    draw(cx, window);
    assert!(dialog_open(cx, window));
    assert_eq!(title(cx, &reports), "Test-notes stopped working");
    let kept = Kept::in_capsule(capsule.path()).pending().remove(0);
    assert_eq!(kept.document["kind"], "app_error");
    assert_eq!(kept.document["component"]["id"], "com.isonomic.test-notes");
    let message = kept.document["message"].as_str().unwrap();
    assert!(
        message.starts_with("Couldn't save the chat list: at ~/Library/x"),
        "{message}"
    );
    assert!(
        message.contains("[key removed]") && !message.contains("iso_tok"),
        "{message}"
    );
}

/// A sender that answers from a script and records what it was given.
struct Scripted {
    answers: std::sync::Mutex<Vec<reports::send::Attempt>>,
    sent: std::sync::Mutex<Vec<(String, String)>>,
}

impl Scripted {
    fn new(answers: Vec<reports::send::Attempt>) -> Arc<Self> {
        Arc::new(Self {
            answers: std::sync::Mutex::new(answers),
            sent: std::sync::Mutex::new(Vec::new()),
        })
    }

    fn sent(&self) -> Vec<(String, String)> {
        self.sent.lock().unwrap().clone()
    }
}

impl reports::send::Sender for Scripted {
    fn send(
        &self,
        body: String,
        id: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = reports::send::Attempt> + Send>> {
        self.sent.lock().unwrap().push((body, id));
        let mut answers = self.answers.lock().unwrap();
        let answer = if answers.len() > 1 {
            answers.remove(0)
        } else {
            answers[0].clone()
        };
        Box::pin(async move { answer })
    }
}

fn with_sender(
    cx: &mut TestAppContext,
    capsule: &std::path::Path,
    sender: Arc<Scripted>,
) -> (AnyWindowHandle, Entity<ProblemReports>) {
    std::fs::write(
        capsule.join("capsule.json"),
        r#"{ "format": 1, "capsule_id": "cap-test" }"#,
    )
    .unwrap();
    let (window, reports, _) = open(cx, capsule);
    reports.update(cx, |r, cx| {
        r.send_with(sender, "reports.example.test".into(), cx)
    });
    draw(cx, window);
    (window, reports)
}

#[ui::test]
fn send_posts_exactly_what_show_report_shows_and_deletes_it(cx: &mut TestAppContext) {
    use reports::send::Attempt;
    let capsule = tempfile::tempdir().unwrap();
    keep_report(capsule.path(), "panic", 1);
    let sender = Scripted::new(vec![Attempt::Sent]);
    let (window, reports) = with_sender(cx, capsule.path(), sender.clone());
    assert!(dialog_open(cx, window));
    assert!(shown(cx, window, "report-send"));
    assert!(shown(cx, window, "report-setting"));
    let shown_bytes = reports.read_with(cx, |r, _| r.outgoing()).unwrap();
    let file = Kept::in_capsule(capsule.path()).pending().remove(0);
    let id = Kept::in_capsule(capsule.path()).report_id(&file);
    assert!(
        shown_bytes.contains(&format!("\"id\": \"{id}\"")),
        "{shown_bytes}"
    );
    assert!(!shown_bytes.contains("isonomic_id"));

    click(cx, window, "report-send");
    assert_eq!(sender.sent(), vec![(shown_bytes, id)]);
    assert_eq!(
        reports.read_with(cx, |r, _| r.last_outcome()),
        Some(shell::problems::SENT)
    );
    assert!(!file.path.exists(), "a 2xx deletes the kept file");
}

#[ui::test]
fn a_network_error_is_tried_again_and_a_4xx_never_is(cx: &mut TestAppContext) {
    use reports::send::{Attempt, FIRST_WAIT};
    let capsule = tempfile::tempdir().unwrap();
    keep_report(capsule.path(), "panic", 1);
    let sender = Scripted::new(vec![
        Attempt::Failed("could not connect".into()),
        Attempt::Sent,
    ]);
    let (window, reports) = with_sender(cx, capsule.path(), sender.clone());
    let file = Kept::in_capsule(capsule.path()).pending().remove(0);
    click(cx, window, "report-send");
    assert_eq!(
        reports.read_with(cx, |r, _| r.last_outcome()),
        Some(shell::problems::NOT_SENT)
    );
    assert!(file.path.exists(), "kept until it goes");
    cx.executor().advance_clock(FIRST_WAIT);
    draw(cx, window);
    assert_eq!(sender.sent().len(), 2);
    assert!(!file.path.exists());

    // A 4xx: kept, and never tried again.
    keep_report(capsule.path(), "hang", 2);
    let refusing = Scripted::new(vec![Attempt::Refused(400)]);
    reports.update(cx, |r, cx| r.send_with(refusing.clone(), "x".into(), cx));
    cx.update_window(window, |_, window, cx| {
        reports.update(cx, |r, cx| r.review(window, cx))
    })
    .unwrap();
    draw(cx, window);
    click(cx, window, "report-send");
    cx.executor().advance_clock(FIRST_WAIT * 4);
    draw(cx, window);
    assert_eq!(refusing.sent().len(), 1);
    assert_eq!(Kept::in_capsule(capsule.path()).pending().len(), 1);
}

#[ui::test]
fn send_automatically_sends_quietly_and_never_the_persons_id(cx: &mut TestAppContext) {
    use reports::send::Attempt;
    let capsule = tempfile::tempdir().unwrap();
    settings::save(capsule.path(), &ReportSettings { mode: Mode::Always }).unwrap();
    let sender = Scripted::new(vec![Attempt::Sent]);
    let (window, reports) = with_sender(cx, capsule.path(), sender.clone());
    reports.update(cx, |r, cx| r.set_identity(Some("ISO-TEST".into()), cx));
    keep_report(capsule.path(), "panic", 1);
    cx.update_window(window, |_, window, cx| {
        reports.update(cx, |r, cx| r.review(window, cx))
    })
    .unwrap();
    draw(cx, window);
    assert!(!dialog_open(cx, window));
    let sent = sender.sent();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("\"id\"") && !sent[0].0.contains("ISO-TEST"));
    assert!(Kept::in_capsule(capsule.path()).pending().is_empty());
}

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
) -> (AnyWindowHandle, Entity<ProblemReports>) {
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
    (window, reports)
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
    let (window, reports) = open(cx, capsule.path());

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
    let never = ReportSettings {
        mode: Mode::Never,
        destination: None,
    };
    settings::save(capsule.path(), &never).unwrap();
    keep_report(capsule.path(), "panic", 1);
    let (window, reports) = open(cx, capsule.path());
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
    let (window, reports) = open(cx, capsule.path());
    reports.update(cx, |r, cx| r.set_identity(Some("ISO-TEST".into()), cx));
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

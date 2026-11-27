// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings: every page, tab, sub-page and dialog of the design draws, the
//! files hold together, and what is set is kept.

extern crate ui as gpui;

use std::sync::Arc;

use isonomic_app::{MemorySettings, SettingsStore};
use serde_json::Value;
use shell::settings::{spec, Settings, APP, KEY};
use ui::component::theme::{Theme, ThemeMode};
use ui::component::WindowExt as _;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, SharedString,
    TestAppContext, WindowBounds, WindowOptions,
};

struct Harness {
    window: AnyWindowHandle,
    view: Entity<Settings>,
    store: Arc<MemorySettings>,
}

fn open_over(cx: &mut TestAppContext, store: Arc<MemorySettings>) -> Harness {
    let kept = Arc::clone(&store);
    let (window, view) = cx.update(|cx| {
        ui::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1376.), px(968.)),
            })),
            ..Default::default()
        };
        ui::open_window(options, cx, |window, cx| {
            cx.new(|cx| Settings::new(kept, window, cx))
        })
        .expect("open test window")
    });
    cx.run_until_parked();
    Harness {
        window,
        view,
        store,
    }
}

fn open(cx: &mut TestAppContext) -> Harness {
    open_over(cx, Arc::new(MemorySettings::default()))
}

/// Draw a frame and say whether the element called `id` is on it.
fn shows(cx: &mut TestAppContext, h: &Harness, id: &str) -> bool {
    let id = SharedString::from(id.to_string());
    cx.update_window(h.window, |_, window, cx| {
        window.render_frame(cx);
        window.try_find(id).is_some_and(|found| found.visible())
    })
    .unwrap()
}

fn is_dialog(id: &str) -> bool {
    spec::spec(id).is_some_and(|of| spec::text(of, "kind") == "dialog")
}

#[test]
fn the_files_hold_together() {
    assert_eq!(spec::faults(), Vec::<String>::new());
    // Every name is a file with blocks in it.
    for id in spec::ids() {
        let of = spec::spec(id).unwrap();
        assert_eq!(spec::text(of, "id"), id);
        let blocks = spec::list(of, "blocks");
        assert!(!blocks.is_empty(), "{id}");
    }
    assert_eq!(
        spec::owner("keyboard-shortcuts-chats"),
        "keyboard-shortcuts"
    );
    assert_eq!(spec::owner("linked-devices"), "profile");
    assert_eq!(
        spec::tab("models", "Available", 1),
        Some("models-available")
    );
    assert_eq!(spec::tab("models", "Installed", 0), Some("models"));
    assert_eq!(spec::slug("Mini & Pets"), "mini-pets");
}

#[ui::test]
fn every_page_draws_in_both_themes(cx: &mut TestAppContext) {
    let h = open(cx);
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        cx.update(|cx| Theme::change(mode, None, cx));
        for id in spec::ids().filter(|id| !is_dialog(id)) {
            h.view.update(cx, |view, cx| view.go(id, cx));
            assert!(
                shows(cx, &h, &format!("settings-page-{id}")),
                "{id} in {mode:?}"
            );
        }
    }
}

#[ui::test]
fn every_dialog_opens_and_closes(cx: &mut TestAppContext) {
    let h = open(cx);
    for id in spec::ids().filter(|id| is_dialog(id)) {
        let open = cx
            .update_window(h.window, |_, window, cx| {
                h.view
                    .update(cx, |view, cx| view.open_dialog(id, window, cx));
                window.render_frame(cx);
                window.render_frame(cx);
                let open = window.has_active_dialog(cx);
                window.close_all_dialogs(cx);
                open
            })
            .unwrap();
        assert!(open, "{id}");
        cx.run_until_parked();
    }
}

#[ui::test]
fn a_button_opens_what_the_design_gives_it(cx: &mut TestAppContext) {
    let h = open(cx);
    let open = cx
        .update_window(h.window, |_, window, cx| {
            h.view.update(cx, |view, cx| {
                view.press("general", "projectless-chat-folder", "Change", window, cx)
            });
            window.render_frame(cx);
            window.has_active_dialog(cx)
        })
        .unwrap();
    assert!(open, "Change opens the folder's dialog");
    // A row with an arrow leads to its own page, and "back" comes back.
    cx.update_window(h.window, |_, window, cx| {
        window.close_all_dialogs(cx);
        h.view.update(cx, |view, cx| {
            view.go("profile", cx);
            view.press("profile", "linked-devices", ">", window, cx);
        });
    })
    .unwrap();
    assert_eq!(
        h.view.read_with(cx, |view, _| view.page().to_string()),
        "linked-devices"
    );
    assert!(shows(cx, &h, "settings-page-linked-devices"));
}

#[ui::test]
fn what_is_set_is_kept(cx: &mut TestAppContext) {
    let h = open(cx);
    let key = "general|start-at-login|0".to_string();
    h.view
        .update(cx, |view, cx| view.set(key.clone(), Value::Bool(true), cx));
    cx.run_until_parked();
    let kept = h.store.get(APP, KEY).expect("the values are kept");
    assert_eq!(kept[key.as_str()], Value::Bool(true));
    // Settings opened again over the same store has the value.
    let again = open_over(cx, Arc::clone(&h.store));
    let value = again
        .view
        .read_with(cx, |view, _| view.value(&key).cloned());
    assert_eq!(value, Some(Value::Bool(true)));
}

#[ui::test]
fn the_search_lists_pages_and_settings(cx: &mut TestAppContext) {
    let hits = spec::search("notif");
    assert!(hits
        .iter()
        .any(|hit| hit.page == "notifications" && hit.under == "Personal"));
    assert!(hits.iter().any(|hit| hit.title == "Notification sound"));
    assert!(spec::search("zzzz").is_empty());
    let h = open(cx);
    assert!(shows(cx, &h, "settings-sidebar"));
}

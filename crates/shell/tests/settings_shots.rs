// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures of Settings, one for each page, tab, sub-page and dialog, to
//! set beside the design. `SETTINGS_SHOTS` names the folder they go in;
//! without it, or without a headless renderer, nothing is taken.

extern crate ui as gpui;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use isonomic_app::MemorySettings;
use shell::settings::{spec, Settings};
use ui::component::WindowExt as _;
use ui::test::TestWindowExt as _;
use ui::{
    px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, HeadlessAppContext, Point,
    WindowBounds, WindowOptions,
};

struct Shots {
    cx: HeadlessAppContext,
    window: AnyWindowHandle,
    view: Entity<Settings>,
    folder: PathBuf,
    taken: usize,
}

impl Shots {
    fn open(folder: PathBuf) -> Option<Shots> {
        let mut cx = std::panic::catch_unwind(|| {
            let platform = ui::platform::current_platform(true);
            HeadlessAppContext::with_platform(
                platform.text_system(),
                Arc::new(ui::assets::AllAssets),
                ui::platform::current_headless_renderer,
            )
        })
        .ok()?;
        cx.update(ui::init);
        let (window, view) = cx
            .update(|cx| {
                let options = WindowOptions {
                    focus: false,
                    show: false,
                    // The design's panel: its frame less the title bar and rail.
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::default(),
                        size: size(px(1376.), px(968.)),
                    })),
                    ..Default::default()
                };
                ui::open_window(options, cx, |window, cx| {
                    cx.new(|cx| Settings::new(Arc::new(MemorySettings::default()), window, cx))
                })
            })
            .ok()?;
        cx.capture_screenshot(window).ok()?;
        Some(Shots {
            cx,
            window,
            view,
            folder,
            taken: 0,
        })
    }

    fn shot(&mut self, name: &str) {
        for _ in 0..6 {
            self.cx.advance_clock(Duration::from_millis(60));
            self.cx.run_until_parked();
            let _ = self
                .cx
                .update_window(self.window, |_, window, cx| window.render_frame(cx));
        }
        let image = self
            .cx
            .update_window(self.window, |_, window, cx| {
                window.render_frame(cx);
                window.render_to_image()
            })
            .expect("the window is open")
            .expect("the window renders");
        self.taken += 1;
        let path = self.folder.join(format!("{:03}-{name}.png", self.taken));
        image.save(&path).expect("the picture saves");
    }
}

fn main() {
    let Some(folder) = std::env::var_os("SETTINGS_SHOTS").map(PathBuf::from) else {
        eprintln!("skipped: SETTINGS_SHOTS names no folder");
        return;
    };
    std::fs::create_dir_all(&folder).expect("the folder can be made");
    let Some(mut shots) = Shots::open(folder) else {
        eprintln!("skipped: no headless renderer here");
        return;
    };
    let only = std::env::var("SETTINGS_SHOTS_ONLY").ok();
    let wanted = |id: &str| only.as_deref().is_none_or(|only| id.contains(only));
    let kind = |id: &str| spec::spec(id).map_or("", |of| spec::text(of, "kind"));
    for id in spec::ids().filter(|id| matches!(kind(id), "page" | "sub")) {
        if wanted(id) {
            let view = shots.view.clone();
            shots
                .cx
                .update(|cx| view.update(cx, |view, cx| view.go(id, cx)));
            shots.shot(id);
        }
    }
    for id in spec::ids().filter(|id| kind(id) == "dialog") {
        if wanted(id) {
            let view = shots.view.clone();
            let _ = shots.cx.update_window(shots.window, |_, window, cx| {
                view.update(cx, |view, cx| view.open_dialog(id, window, cx));
            });
            shots.shot(id);
            let _ = shots
                .cx
                .update_window(shots.window, |_, window, cx| window.close_all_dialogs(cx));
        }
    }
    eprintln!("{} pictures", shots.taken);
}

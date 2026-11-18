// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The first thing the window shows (design "01 · App launch"): the page,
//! empty, with the mark in its centre playing its intro once. When the
//! intro has ended and been held for a moment, the shell moves on.
//!
//! The intro is drawn from the clock at each frame; nothing polls, and a
//! frame is asked for only while the mark is still moving.

use std::time::Instant;

use ui::component::theme::ActiveTheme as _;
use ui::{
    canvas, div, logo, tokens, AppContext as _, Context, Entity, EventEmitter,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _,
    TestSupportExt as _, Window,
};

use crate::Shell;

/// The launch screen is over; the shell shows what comes next.
pub struct Finished;

#[derive(Default)]
pub struct Launch {
    /// When the first frame was drawn: the intro's clock starts there, so
    /// a window that is slow to appear doesn't swallow it.
    started: Option<Instant>,
    finished: bool,
}

impl EventEmitter<Finished> for Launch {}

impl Launch {
    pub fn new() -> Self {
        Self::default()
    }

    /// End the launch screen now. Once only.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        if !self.finished {
            self.finished = true;
            cx.emit(Finished);
        }
    }

    /// The intro's frame now; zero before the first draw.
    pub fn frame(&self) -> f32 {
        self.started
            .map_or(0., |started| logo::frame_at(started.elapsed()))
    }
}

impl Render for Launch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.started.is_none() {
            self.started = Some(Instant::now());
            cx.spawn(async move |view, cx| {
                cx.background_executor()
                    .timer(logo::duration() + tokens::launch_hold())
                    .await;
                let _ = view.update(cx, |view, cx| view.finish(cx));
            })
            .detach();
        }
        let frame = self.frame();
        if frame < logo::LAST_FRAME {
            window.request_animation_frame();
        }
        let color = logo::mark_color();
        div()
            .id("launch")
            .test_support()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().background)
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| logo::paint(bounds, frame, color, window),
                )
                .size(tokens::logo_size()),
            )
    }
}

impl Shell {
    /// Open on the launch screen: the mark plays its intro once, then the
    /// window shows first run or its tabs.
    pub fn start_launch(&mut self, cx: &mut Context<Self>) {
        let launch = cx.new(|_| Launch::new());
        cx.subscribe(&launch, |shell, _, _: &Finished, cx| {
            shell.launch = None;
            cx.notify();
        })
        .detach();
        self.launch = Some(launch);
        cx.notify();
    }

    /// The launch screen, while it is showing.
    pub fn launch(&self) -> Option<&Entity<Launch>> {
        self.launch.as_ref()
    }
}

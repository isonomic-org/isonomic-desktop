// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The onboarding window's drawing: every piece placed where the design
//! places it in the 1280 × 800 page, moved by the one clock.

use ui::assets::IconName;
use ui::component::button::Button;
use ui::component::input::{InputEvent, InputState};
use ui::component::link::Link;
use ui::component::Icon;
use ui::onboarding::terms::at as third;
use ui::onboarding::{self as design, at};
use ui::AppContext as _;
use ui::{
    canvas, div, img, logo, motion, point, px, Context, FontWeight, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Render, Styled as _, TestSupportExt as _, Window,
};

use super::{kit_run, laptop, terms, Chosen, Onboarding};

/// How far below its place something starts so that it is under the
/// window's bottom edge.
fn below(top: f32) -> f32 {
    design::HEIGHT - top + 8.
}

/// How far something that is leaving has gone, `t` seconds after the
/// press on "Start new account"; nothing before it.
fn departed(t: Option<f32>) -> f32 {
    t.map_or(0., |t| {
        motion::ease(motion::DEPART, motion::progress(t, 0., third::LEAVE))
    })
}

/// The same for "Start new account", which leaves a step later.
fn departed_after(t: Option<f32>) -> f32 {
    departed(t.map(|t| t - third::LEAVE_STAGGER))
}

impl Onboarding {
    /// "Start new account", which leaves to the right when pressed.
    fn new_account(&self, now: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let (wide, tall) = design::BUTTON;
        let (pad_left, pad_right) = design::BUTTON_PAD;
        let top = design::BUTTON_TOP;
        let risen = motion::ease(
            motion::ARRIVE,
            motion::progress(now, at::BUTTONS_RISE, at::RISE),
        );
        let gone = departed_after(self.terms_now());
        let ring = div()
            .flex_none()
            .size(px(design::RING))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border(px(design::RING_LINE))
            .border_color(design::hairline())
            .child(Icon::from(IconName::ArrowRight).size(px(design::ARROW)));
        div()
            .absolute()
            .left(px(
                design::BUTTON_LEFT + (design::WIDTH - design::BUTTON_LEFT + 8.) * gone
            ))
            .top(px(top + below(top) * (1. - risen)))
            // A press here is a press, not the start of moving the window.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                Button::new("onboarding-new-account")
                    .outline()
                    .w(px(wide))
                    .h(px(tall))
                    .pl(px(pad_left))
                    .pr(px(pad_right))
                    .rounded(px(tall / 2.))
                    .shadow(design::button_glow())
                    .on_click(cx.listener(|view, _, _, cx| view.choose(Chosen::NewAccount, cx)))
                    .child(
                        div()
                            .w(px(wide - pad_left - pad_right - 2.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(design::BUTTON_LABEL))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Start new account")
                            .child(ring),
                    ),
            )
    }

    /// "Recover existing account", a link under it, which drops out of
    /// the window when a new account is started.
    fn recover(&self, now: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let (left, _, wide, _) = design::TITLE;
        let (size, line) = design::LINK_LABEL;
        let top = design::LINK_TOP;
        let risen = motion::ease(
            motion::ARRIVE,
            motion::progress(now, at::BUTTONS_RISE + at::BUTTON_STAGGER, at::RISE),
        );
        div()
            .id("onboarding-recover")
            .test_support()
            .absolute()
            .left(px(left))
            .top(px(top
                + below(top) * (1. - risen)
                + below(top) * departed(self.terms_now())))
            .w(px(wide))
            .flex()
            .justify_center()
            .child(
                Link::new("onboarding-recover-link")
                    .text_size(px(size))
                    .line_height(px(line))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(design::ink())
                    .on_click(cx.listener(|view, _, _, cx| view.choose(Chosen::Recover, cx)))
                    .child("Recover existing account"),
            )
    }
}

impl Render for Onboarding {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.started.is_none() {
            self.start(cx);
        }
        self.scale = window.scale_factor();
        if self.search.is_none() {
            self.search = Some([0, 1].map(|_| {
                let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
                // What is sought changes what its list shows.
                cx.subscribe_in(&search, window, |_, _, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                })
                .detach();
                search
            }));
        }
        self.land();
        self.draw_kit(cx);
        if self.words.is_none() {
            self.words = Some([0, 1, 2].map(|index| {
                let word = cx.new(|cx| InputState::new(window, cx).validate(kit_run::fits));
                cx.subscribe_in(
                    &word,
                    window,
                    move |view, word, event: &InputEvent, _, cx| {
                        view.word_event(index, word.read(cx).value(), event, cx)
                    },
                )
                .detach();
                word
            }));
        }
        if let Some(words) = &self.words {
            if std::mem::take(&mut self.words_stale) {
                for (word, typed) in words.iter().zip(&self.typed) {
                    if word.read(cx).value() != typed.as_str() {
                        word.update(cx, |word, cx| word.set_value(typed.clone(), window, cx));
                    }
                }
            }
            if std::mem::take(&mut self.wants_words) {
                words[self.typing].update(cx, |word, cx| word.focus(window, cx));
            }
        }
        let now = self.now();
        let third_now = self.terms_now();
        // The third screen's dotted line never stops pulsing.
        if now < at::END || third_now.is_some() {
            window.request_animation_frame();
        }

        // The mark: its intro, then the turn into the corner.
        // The intro starts with the sound's first note.
        let frame = logo::frame_at(std::time::Duration::from_secs_f32(
            (now - at::MARK_STARTS).max(0.),
        ));
        let gone = motion::ease(
            motion::CARRY,
            motion::progress(now, at::MARK_LEAVES, at::MARK_TRAVEL),
        );
        let mark = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let (rest_x, rest_y) = design::MARK_REST_CENTRE;
                let centre = point(
                    bounds.origin.x + px(motion::between(design::WIDTH / 2., rest_x, gone)),
                    bounds.origin.y + px(motion::between(design::HEIGHT / 2., rest_y, gone)),
                );
                logo::paint_at(
                    centre,
                    px(motion::between(design::MARK_INTRO, design::MARK_REST, gone)),
                    design::MARK_SPIN * gone,
                    frame,
                    design::ink(),
                    window,
                );
            },
        )
        .absolute()
        .size_full();

        // The laptop: always its 3D model, as its latest drawn frame on a
        // larger canvas, and placed by the time that frame was drawn for. It
        // flies up on the second screen, across to the sidebar on the
        // third, and rests as its last frame.
        let margin = (design::LAPTOP_CANVAS - 1.) / 2.;
        let own_laptop = self.frame.as_ref().filter(|_| !self.is_setting_up());
        let laptop = own_laptop.map(|frame| {
            let ((left, top, wide, tall), drop) = match third_now {
                None => {
                    let (landed, _, _) = laptop::flight(frame.at);
                    let (_, top, _, tall) = design::LAPTOP;
                    (design::LAPTOP, (below(top) + tall * margin) * (1. - landed))
                }
                Some(_) => (terms::laptop_box(terms::laptop_move(frame.at).0), 0.),
            };
            img(frame.image.clone())
                .id("onboarding-laptop")
                .absolute()
                .left(px(left - wide * margin))
                .top(px(top - tall * margin + drop))
                .w(px(wide * design::LAPTOP_CANVAS))
                .h(px(tall * design::LAPTOP_CANVAS))
        });

        let (title_left, title_top, title_wide, title_tall) = design::TITLE;
        let title_risen = motion::ease(
            motion::ARRIVE,
            motion::progress(now, at::TITLE_RISES, at::RISE),
        );
        let title = div()
            .id("onboarding-title")
            .test_support()
            .absolute()
            .left(px(title_left))
            .top(px(title_top
                + below(title_top) * (1. - title_risen)
                + below(title_top) * departed(third_now)))
            .w(px(title_wide))
            .h(px(title_tall))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(design::TITLE_SIZE))
            .line_height(px(title_tall))
            .font_weight(FontWeight::MEDIUM)
            .text_color(design::ink())
            .child("Welcome to Isonomic");

        let new_account = self.new_account(now, cx);
        let recover = self.recover(now, cx);

        div()
            .id("onboarding")
            .test_support()
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(design::corner())
            .bg(design::page())
            // With no title bar, the page itself moves the window.
            .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
            .children(third_now.map(|t| self.terms_backdrop(t)))
            .child(mark)
            .children(laptop)
            .child(title)
            .child(new_account)
            .child(recover)
            .children(third_now.map(|t| self.terms_front(t, cx)))
    }
}

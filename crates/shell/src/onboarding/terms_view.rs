// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The third screen's drawing: the sidebar and its dotted line behind the
//! mark and the laptop, and in front of them the stepper, the content and
//! the two props, each placed where the design places it.

use ui::component::Icon;
use ui::onboarding::passkey::at as fourth;
use ui::onboarding::terms::{self as design, at, signs};
use ui::onboarding::{self as page};
use ui::{
    div, motion, px, Context, FontWeight, HighlightStyle, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    StyledText, TestSupportExt as _,
};

use super::{terms, Onboarding};

/// The disclosure's clauses: the words in heavier type, then the rest.
const CLAUSES: [(&str, &str); 9] = [
    ("Early-stage software.", " Isonomic Desktop, its engine and its apps are new and still in active development. They may contain errors that cause data loss, interrupted service or unexpected behaviour. Keep your own backups."),
    ("Your keys, your responsibility.", " Your account is secured by a passkey on this Mac and a recovery kit you keep. Anud Inc. cannot see, reset or recover your keys. If you lose both your devices and your recovery kit, your data and licences cannot be restored."),
    ("Local AI models.", " Models run on this machine and may produce inaccurate, incomplete or offensive output. Do not rely on them for medical, legal, financial or safety decisions. Model downloads use disk space and memory."),
    ("Payments and licences.", " App licences are paid in USDC and recorded on a public network. Network transactions are final once confirmed. The value of digital assets can change, and card payments go through a third-party onramp with its own terms and fees."),
    ("Third-party connections.", " Apps and integrations you connect act with the access you grant them. Review what each one can read and send before you allow it."),
    ("Availability and updates.", " Isonomic may change, pause or end features and apps. Updates can change how your data is stored; the engine migrates it, but you should keep a recent backup before you update."),
    ("Backups and storage.", " Backups you choose to keep in iCloud or another cloud provider are stored under that provider\u{2019}s terms. Encrypted backups cannot be opened without your keys."),
    ("Regulatory change.", " Laws on digital assets, AI and data protection are changing. New rules could limit how Isonomic, its app store or its payments work where you live."),
    ("No advice.", " Nothing in Isonomic or its apps is financial, legal or tax advice."),
];
const PREAMBLE: &str = "Anud Inc. makes Isonomic available for your information and use at your own risk. You confirm that you understand the following risks, and that this list may not contain every risk.";
const CERTIFY: &str = "By clicking \u{201c}Accept and continue\u{201d} you certify that you have read and accept the Risk disclosure and the Terms of Service.";

/// How far below its place something starts so that it is under the
/// window's bottom edge.
fn below(top: f32) -> f32 {
    page::HEIGHT - top + 8.
}

/// How far right of its place something starts so that it is past the
/// window's right edge.
fn beyond(left: f32) -> f32 {
    page::WIDTH - left + 8.
}

impl Onboarding {
    /// Behind the mark and the laptop: the sidebar's colour and the dotted
    /// line from the laptop down to the clipboard.
    pub(super) fn terms_backdrop(&self, t: f32) -> impl IntoElement {
        let shown = motion::progress(t, at::SIDEBAR_SHOWS, at::SIDEBAR_FADE);
        let circle = |y: f32, radius: f32, solid: f32| {
            div()
                .absolute()
                .left(px(design::DOTS_X - radius))
                .top(px(y - radius))
                .size(px(radius * 2.))
                .rounded(px(radius))
                .bg(design::dot().opacity(solid))
        };
        // The line goes as the clipboard leaves.
        let fading = self.signing_at(t).map_or(1., |s| {
            1. - motion::progress(s, signs::CLIPBOARD_LEAVES, signs::DOTS_FADE)
        });
        // And comes back once the seal has landed.
        let fading = fading
            + self.passkey_at(t).map_or(0., |p| {
                motion::progress(p, fourth::LIGHT_SHOWS, fourth::LIGHT_FADE)
            });
        let port_shown = motion::progress(t, at::DOTS_START, at::DOT_SHOWS) * fading;
        let dots = (0..design::DOTS).map(|index| {
            let (solid, radius) = terms::dot(t, index);
            let solid = solid * fading;
            circle(
                design::DOTS_TOP + design::DOTS_STEP * index as f32,
                radius,
                solid,
            )
        });
        div()
            .absolute()
            .size_full()
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .w(px(design::SIDEBAR))
                    .h_full()
                    .rounded_tl(page::corner())
                    .rounded_bl(page::corner())
                    .border_r_1()
                    .border_color(page::hairline())
                    .bg(design::sidebar())
                    .opacity(shown),
            )
            .child(circle(design::DOTS_TOP, design::PORT, port_shown))
            .children(dots)
    }

    fn stepper(&self, t: f32) -> impl IntoElement {
        let (left, top, wide) = design::STEPPER;
        let risen = motion::ease(
            motion::ARRIVE,
            motion::progress(t, at::STEPPER_RISES, at::RISE),
        );
        let (label, label_line) = design::STEP_LABEL;
        let (title, title_line) = design::STEP_TITLE;
        // On the fourth screen the words change: out, then in.
        let change = self
            .passkey_at(t)
            .map_or(0., |p| motion::progress(p, 0., fourth::WORDS_CHANGE));
        let (title_words, words) = if change < 0.5 {
            (
                "Accept terms",
                "Isonomic runs on your Mac and you hold your own keys.",
            )
        } else {
            (
                "Create account",
                "Face ID creates your account. No seed phrase, no password.",
            )
        };
        let words_solid = (1. - 2. * change).abs();
        let bars = (0..design::STEPS).map(|step| {
            div()
                .flex_1()
                .h(px(design::STEP_BAR))
                .rounded(px(design::STEP_BAR / 2.))
                .bg(if step == 0 {
                    page::ink()
                } else {
                    design::outline()
                })
        });
        div()
            .id("onboarding-stepper")
            .test_support()
            .absolute()
            .left(px(left))
            .top(px(top + below(top) * (1. - risen)))
            .w(px(wide))
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(
                div()
                    .w(px(design::STEPPER_TEXT))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .text_size(px(label))
                    .line_height(px(label_line))
                    .text_color(design::quiet())
                    .opacity(words_solid)
                    .child(
                        div()
                            .font_family(design::MONO)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("STEP 1"),
                    )
                    .child(
                        div()
                            .text_size(px(title))
                            .line_height(px(title_line))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(page::ink())
                            .child(title_words),
                    )
                    .child(words),
            )
            .child(div().w_full().flex().gap(px(8.)).children(bars))
    }

    /// A footer button: its label with an arrow before or after it.
    /// A footer button: back, or forward, and what a press does.
    pub(super) fn action(
        &self,
        id: &'static str,
        label: &'static str,
        forward: bool,
        press: fn(&mut Self, &mut Context<Self>),
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (near, far, above) = design::ACTION_PAD;
        let (size, line) = design::ACTION_LABEL;
        let (ink, arrow) = if forward {
            (page::page(), "icons/arrow-right.svg")
        } else {
            (design::body(), "icons/arrow-left.svg")
        };
        let arrow = Icon::default()
            .path(arrow)
            .size(px(design::ACTION_ARROW))
            .text_color(ink);
        let button = div()
            .id(id)
            .test_support()
            .flex()
            .items_center()
            .gap(px(8.))
            .py(px(above))
            .rounded(px(design::ACTION_CORNER))
            .border_1()
            .shadow(design::action_shadow())
            .cursor_pointer()
            .text_size(px(size))
            .line_height(px(line))
            .font_weight(FontWeight::MEDIUM)
            .text_color(ink)
            // A press here is a press, not the start of moving the window.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |view, _, _, cx| press(view, cx)));
        if forward {
            button
                .pl(px(far))
                .pr(px(near))
                .border_color(page::ink())
                .bg(page::ink())
                .child(label)
                .child(arrow)
        } else {
            button
                .pl(px(near))
                .pr(px(far))
                .border_color(design::outline())
                .bg(page::page())
                .child(arrow)
                .child(label)
        }
    }

    fn content(&self, t: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let slid = motion::ease(
            motion::ARRIVE,
            motion::progress(t, at::CONTENT_SLIDES, at::SLIDE),
        );
        let (heading, heading_line) = design::HEADING;
        let (sub, sub_line) = design::SUBHEADING;
        let (clause_title, clause_title_line) = design::CLAUSE_TITLE;
        let (clause, clause_line) = design::CLAUSE;
        let (header_top, header_bottom) = design::HEADER_PAD;
        let (disclosure_top, disclosure_tall) = design::DISCLOSURE;
        let (pad_top, pad_bottom) = design::DISCLOSURE_PAD;
        let clauses = CLAUSES.iter().map(|(lead, rest)| {
            let heavier = HighlightStyle {
                font_weight: Some(FontWeight::SEMIBOLD),
                ..Default::default()
            };
            div().flex_shrink_0().child(
                StyledText::new(SharedString::from(format!("{lead}{rest}")))
                    .with_highlights([(0..lead.len(), heavier)]),
            )
        });
        div()
            .id("onboarding-terms")
            .test_support()
            .absolute()
            .left(px(design::SIDEBAR + beyond(design::SIDEBAR) * (1. - slid)))
            .top_0()
            .w(px(page::WIDTH - design::SIDEBAR))
            .h_full()
            // Opaque: the sidebar's props start hidden under it.
            .rounded_tr(page::corner())
            .rounded_br(page::corner())
            .bg(page::page())
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .w_full()
                    .pt(px(header_top))
                    .pb(px(header_bottom))
                    .px(px(design::PAD))
                    .border_b_1()
                    .border_color(page::hairline())
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(heading))
                            .line_height(px(heading_line))
                            .font_weight(FontWeight::BOLD)
                            .text_color(page::ink())
                            .child("Before you continue"),
                    )
                    .child(
                        div()
                            .text_size(px(sub))
                            .line_height(px(sub_line))
                            .text_color(design::quiet())
                            .child(CERTIFY),
                    ),
            )
            .child(
                div()
                    .id("onboarding-disclosure")
                    .absolute()
                    .left_0()
                    .top(px(disclosure_top))
                    .w_full()
                    .h(px(disclosure_tall))
                    .overflow_y_scroll()
                    .pt(px(pad_top))
                    .pb(px(pad_bottom))
                    .px(px(design::PAD))
                    .flex()
                    .flex_col()
                    .gap(px(design::CLAUSE_GAP))
                    .font_family(design::SERIF)
                    .text_size(px(clause))
                    .line_height(px(clause_line))
                    .text_color(design::body())
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(clause_title))
                            .line_height(px(clause_title_line))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(page::ink())
                            .child("Risk disclosure"),
                    )
                    .child(div().flex_shrink_0().child(PREAMBLE))
                    .children(clauses),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .bottom_0()
                    .w_full()
                    .h(px(design::FOOTER))
                    .px(px(design::FOOTER_PAD))
                    .border_t_1()
                    .border_color(page::hairline())
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(self.action(
                        "onboarding-decline",
                        "Decline",
                        false,
                        Onboarding::decline,
                        cx,
                    ))
                    .child(self.action(
                        "onboarding-accept",
                        "Accept and continue",
                        true,
                        Onboarding::accept,
                        cx,
                    )),
            )
    }

    /// In front of the mark and the laptop: the stepper, the clipboard and
    /// the pen, and over them the content, which they come out from under;
    /// then the fourth screen's seal and, over it, its panel.
    pub(super) fn terms_front(&self, t: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let fourth = self.passkey_at(t);
        div()
            .absolute()
            .size_full()
            .child(self.stepper(t))
            .child(self.props_layer(t))
            .child(self.content(t, cx))
            .children(fourth.map(|p| self.seal_layer(p)))
            .children(fourth.map(|p| self.passkey_panel(p, cx)))
    }
}

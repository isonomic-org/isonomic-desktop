// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The third screen's props as drawn: the clipboard, the signature being
//! written on its last page, and the pen. Each drawn frame is placed by
//! the time it was drawn for.

use ui::onboarding::terms::{self as design, at, signs};
use ui::{
    canvas, div, img, motion, point, px, IntoElement, ParentElement as _, PathBuilder, Styled as _,
};

use super::{signature, signing, terms, Onboarding};

impl Onboarding {
    /// The clipboard, the signature and the pen, at `t` on the third
    /// screen's clock.
    pub(super) fn props_layer(&self, t: f32) -> impl IntoElement {
        let (left, top, wide, tall) = design::CLIPBOARD;
        let margin = (design::CLIPBOARD_CANVAS - 1.) / 2.;
        let clipboard = self.clipboard.as_ref().map(|frame| {
            let (arrived, _) = terms::flight(frame.at, at::CLIPBOARD_FLIES);
            // It comes out from under the content, and leaves by the left
            // edge.
            let (gone, _) = self.signing_at(frame.at).map_or((0., 0.), |s| {
                signing::leaving(s, signs::CLIPBOARD_LEAVES, signs::LEAVE)
            });
            let from = design::SIDEBAR + design::TUCK;
            img(frame.image.clone())
                .absolute()
                .left(px(motion::between(from, left, arrived)
                    - wide * margin
                    - (left + wide) * gone))
                .top(px(top - tall * margin))
                .w(px(wide * design::CLIPBOARD_CANVAS))
                .h(px(tall * design::CLIPBOARD_CANVAS))
        });

        // The signature: as much as is written, while its page is in view.
        let written = self
            .signing_at(t)
            .filter(|_| {
                let drawn_for = self.clipboard.as_ref().map_or(t, |frame| frame.at);
                self.signing_at(drawn_for).is_some_and(signing::uncovered)
            })
            .map(signing::written)
            .filter(|share| *share > 0.);
        let signed = written.zip(self.props.clone()).map(|(share, props)| {
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let place = |on_line: (f32, f32)| {
                        let (x, y) = props.signature_point(on_line);
                        point(
                            bounds.origin.x + px(left + wide / 2. + x),
                            bounds.origin.y + px(top + tall / 2. + y),
                        )
                    };
                    let points = signature::written(share);
                    let mut line = PathBuilder::stroke(px(design::SIGNATURE_LINE));
                    line.move_to(place(points[0]));
                    for on_line in &points[1..] {
                        line.line_to(place(*on_line));
                    }
                    if let Ok(line) = line.build() {
                        window.paint_path(line, design::signature());
                    }
                },
            )
            .absolute()
            .size_full()
        });

        let (left, top, wide, tall) = design::PEN;
        let pen = self.pen.as_ref().map(|frame| {
            let (arrived, _) = terms::flight(frame.at, at::PEN_FLIES);
            let (gone, _) = self.signing_at(frame.at).map_or((0., 0.), |s| {
                signing::leaving(s, signs::PEN_LEAVES, signs::PEN_LEAVE)
            });
            // Its picture is still while it writes, so it goes by the clock.
            let (reach_x, reach_y) = self.signing_at(t).map_or((0., 0.), |s| self.pen_reach(s));
            let rest = left + (wide - design::PEN_CANVAS) / 2.;
            img(frame.image.clone())
                .absolute()
                .left(px(motion::between(design::SIDEBAR, rest, arrived)
                    + reach_x
                    - (rest + design::PEN_CANVAS) * gone))
                .top(px(top + (tall - design::PEN_CANVAS) / 2. + reach_y))
                .size(px(design::PEN_CANVAS))
        });
        div()
            .absolute()
            .size_full()
            .children(clipboard)
            .children(signed)
            .children(pen)
    }
}

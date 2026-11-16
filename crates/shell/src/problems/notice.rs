//! The notice and its review, as Isonomic UI dialogs. The builders run
//! again whenever the dialog layer redraws, so they only read the entity.

use ui::base::{h_flex, v_flex};
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::checkbox::Checkbox;
use ui::component::dialog::DialogFooter;
use ui::component::theme::ActiveTheme as _;
use ui::component::WindowExt as _;
use ui::prelude::FluentBuilder as _;
use ui::{
    div, tokens, App, Entity, InteractiveElement as _, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

use super::{ProblemReports, ID_LABEL, KEPT_HERE, PURPOSE, SEND_OR_KEEP};

/// Open the notice for the report `reports` is on.
pub(super) fn open(reports: Entity<ProblemReports>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, cx| {
        let state = reports.read(cx);
        if state.current().is_none() {
            return dialog;
        }
        let muted = cx.theme().muted_foreground;
        let checkbox = state.offers_id().then(|| {
            let reports = reports.clone();
            Checkbox::new("report-include-id")
                .label(ID_LABEL)
                .checked(state.includes_id())
                .on_click(move |_, _, cx| reports.update(cx, |r, cx| r.toggle_id(cx)))
        });
        let (keep, gone, show) = (reports.clone(), reports.clone(), reports.clone());
        let offers_send = state.offers_send();
        let where_it_goes = if offers_send { SEND_OR_KEEP } else { KEPT_HERE };
        let send = offers_send.then(|| {
            let send = reports.clone();
            Button::new("report-send")
                .primary()
                .label("Send report")
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    send.update(cx, |r, cx| r.send_current(window, cx));
                })
        });
        let on_close = reports.clone();
        dialog
            .title(state.title())
            .child(
                v_flex()
                    .id("report-notice")
                    .test_support()
                    .gap_3()
                    .child(PURPOSE)
                    .child(div().text_color(muted).child(where_it_goes))
                    .children(checkbox),
            )
            .footer(
                DialogFooter::new()
                    .child(
                        Button::new("report-show")
                            .ghost()
                            .label("Show report")
                            .on_click(move |_, window, cx| review(show.clone(), window, cx)),
                    )
                    .child(
                        Button::new("report-dont-send")
                            .label("Don't send")
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                gone.update(cx, |r, cx| r.dont_send(window, cx));
                            }),
                    )
                    .child(
                        Button::new("report-keep")
                            .when(!offers_send, |b| b.primary())
                            .label("Keep on this Mac")
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                keep.update(cx, |r, cx| r.keep(window, cx));
                            }),
                    )
                    .children(send),
            )
            // Closing it any other way keeps the report; after a button
            // has already acted, there is nothing left to keep.
            .on_close(move |_, window, cx| on_close.update(cx, |r, cx| r.keep(window, cx)))
    });
}

/// Open, over the notice, exactly the bytes that would be sent.
fn review(reports: Entity<ProblemReports>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, _, cx| {
        let bytes = reports.read(cx).outgoing().unwrap_or_default();
        let mono = cx.theme().mono_font_family.clone();
        let lines = bytes.lines().map(|line| {
            div()
                .font_family(mono.clone())
                .text_sm()
                .child(SharedString::from(line.to_string()))
        });
        dialog
            .title("Problem report")
            .w(tokens::content_width())
            .child(
                v_flex()
                    .id("report-bytes")
                    .test_support()
                    .h(tokens::report_bytes_height())
                    .overflow_y_scroll()
                    .children(lines),
            )
            .footer(
                DialogFooter::new().child(
                    h_flex().child(
                        Button::new("report-back")
                            .label("Back")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
                ),
            )
    });
}

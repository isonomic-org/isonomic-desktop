//! Acknowledgements: the licence notices of what the app bundles, as the
//! engine wrote them, in a dialog. The window reads the file; this only
//! shows the text it is given.

use ui::base::v_flex;
use ui::component::alert::Alert;
use ui::component::button::Button;
use ui::component::dialog::DialogFooter;
use ui::component::theme::ActiveTheme as _;
use ui::component::WindowExt as _;
use ui::{
    div, tokens, App, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

/// What a build without the notices says (a development build run
/// outside the app bundle).
pub const MISSING: &str = "This copy of Isonomic has no acknowledgements file. A released app \
carries it inside the app.";

/// Open the dialog with `notices`, or say they are missing.
pub fn open(notices: Option<String>, window: &mut Window, cx: &mut App) {
    let lines: Vec<SharedString> = notices
        .as_deref()
        .map(|text| {
            text.lines()
                .map(|l| SharedString::from(l.to_string()))
                .collect()
        })
        .unwrap_or_default();
    window.open_dialog(cx, move |dialog, _, cx| {
        let body = if lines.is_empty() {
            div()
                .id("acknowledgements-missing")
                .test_support()
                .child(Alert::info("acknowledgements-missing-alert", MISSING))
                .into_any_element()
        } else {
            let mono = cx.theme().mono_font_family.clone();
            v_flex()
                .id("acknowledgements-text")
                .test_support()
                .h(tokens::report_bytes_height())
                .overflow_y_scroll()
                .children(lines.iter().map(|line| {
                    div()
                        .font_family(mono.clone())
                        .text_sm()
                        .child(line.clone())
                }))
                .into_any_element()
        };
        dialog
            .title("Acknowledgements")
            .centered()
            .w(tokens::content_width())
            .child(body)
            .footer(
                DialogFooter::new().child(
                    Button::new("acknowledgements-done")
                        .label("Done")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                ),
            )
    });
}

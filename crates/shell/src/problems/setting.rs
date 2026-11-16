//! The "Problem reports" row on Home, until the window has a Settings tab.

use reports::settings::Mode;
use ui::base::v_flex;
use ui::component::group_box::GroupBox;
use ui::component::label::Label;
use ui::component::radio::{Radio, RadioGroup};
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::{
    div, App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    Styled as _, TestSupportExt as _,
};

use super::{ProblemReports, KEPT_HERE};

const MODES: [(Mode, &str); 3] = [
    (Mode::Ask, "Ask each time"),
    (Mode::Always, "Send automatically"),
    (Mode::Never, "Never send"),
];

pub fn row(reports: &Entity<ProblemReports>, cx: &App) -> impl IntoElement {
    let mode = reports.read(cx).mode();
    let line: SharedString = match reports.read(cx).host() {
        Some(host) => format!("Reports you send go to {host}.").into(),
        None => KEPT_HERE.into(),
    };
    let selected = MODES.iter().position(|(m, _)| *m == mode);
    let reports = reports.clone();
    let radios = MODES
        .iter()
        .map(|(m, label)| Radio::new(format!("report-mode-{}", m.as_str())).label(*label));
    let group = GroupBox::new()
        .id("report-setting-group")
        .title(Label::new("Problem reports").font_semibold())
        .child(
            v_flex()
                .gap_2()
                .child(
                    RadioGroup::horizontal("report-mode")
                        .children(radios)
                        .selected_index(selected)
                        .on_change(move |index, _, cx| {
                            if let Some((mode, _)) = MODES.get(*index) {
                                reports.update(cx, |r, cx| r.set_mode(*mode, cx));
                            }
                        }),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(line),
                ),
        );
    div().id("report-setting").test_support().child(group)
}

//! First run's layout: a stepper across the three steps, one group per
//! step, and the person's actions at the foot. Isonomic UI's components as
//! built, its spacing and type, both themes, every action a button.

use ui::base::{h_flex, v_flex};
use ui::component::alert::Alert;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::group_box::GroupBox;
use ui::component::label::Label;
use ui::component::progress::Progress;
use ui::component::spinner::Spinner;
use ui::component::stepper::{Stepper, StepperItem};
use ui::component::theme::ActiveTheme as _;
use ui::component::{Disableable as _, Sizable as _, StyledExt as _};
use ui::{
    div, tokens, Context, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

use super::{FirstRun, Memory, ModelStep};

/// A line tests can find by `id`.
fn line(id: &'static str, text: impl Into<SharedString>) -> impl IntoElement {
    div().id(id).test_support().child(text.into())
}

impl FirstRun {
    /// Which step the person is on: the memory check until it passes, then
    /// the model.
    fn current_step(&self) -> usize {
        match (&self.memory, &self.model) {
            (Memory::Enough(_), ModelStep::Installed(_)) => 3,
            (Memory::Enough(_), _) => 2,
            _ => 1,
        }
    }

    /// The download's share done, when the engine knows its size.
    fn fraction(&self) -> Option<f32> {
        match &self.model {
            ModelStep::Downloading {
                received,
                total: Some(total),
                ..
            } if *total > 0 => Some((*received as f64 / *total as f64 * 100.0) as f32),
            _ => None,
        }
    }
}

impl Render for FirstRun {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let capsule = if self.created {
            "Your Capsule was created on this Mac. It keeps everything Isonomic stores."
        } else {
            "Your Capsule is on this Mac. It keeps everything Isonomic stores."
        };
        let (memory_id, memory_text) = self.memory_line();

        let mut memory = GroupBox::new()
            .id("first-run-memory")
            .title(Label::new("Memory").font_semibold())
            .child(
                h_flex()
                    .gap_2()
                    .children(
                        matches!(self.memory, Memory::Checking).then(|| Spinner::new().small()),
                    )
                    .child(line(memory_id, memory_text)),
            );
        match &self.memory {
            Memory::TooLittle(_) => {
                memory = memory.child(Alert::error(
                    "memory-too-little-alert",
                    "Local models need more memory than this Mac has, so Isonomic can't finish setting up here.",
                ));
            }
            Memory::Failed(_) => {
                memory = memory.child(
                    h_flex().child(
                        Button::new("memory-check-again")
                            .label("Check again")
                            .on_click(cx.listener(|view, _, _, cx| view.check_memory(cx))),
                    ),
                );
            }
            _ => {}
        }

        let mut model = GroupBox::new()
            .id("first-run-model")
            .title(Label::new("Local model").font_semibold());
        if let Some((id, text)) = self.model_line() {
            model = model.child(
                h_flex()
                    .gap_2()
                    .children(
                        matches!(self.model, ModelStep::Choosing).then(|| Spinner::new().small()),
                    )
                    .child(line(id, text)),
            );
        } else {
            model = model.child(
                div()
                    .text_color(muted)
                    .child("Chosen once the memory check passes."),
            );
        }
        match &self.model {
            ModelStep::Ready(_) => {
                model = model.child(
                    h_flex().child(
                        Button::new("model-download")
                            .primary()
                            .label("Download")
                            .on_click(cx.listener(|view, _, _, cx| view.start_download(cx))),
                    ),
                );
            }
            ModelStep::Downloading { .. } => {
                let progress = match self.fraction() {
                    Some(value) => Progress::new("model-progress").value(value),
                    None => Progress::new("model-progress").loading(true),
                };
                model = model
                    .child(progress.accessibility_label("Model download"))
                    .child(
                        h_flex().child(
                            Button::new("model-cancel")
                                .label("Cancel download")
                                .on_click(cx.listener(|view, _, _, cx| view.cancel_download(cx))),
                        ),
                    );
            }
            ModelStep::Failed { .. } => {
                model = model.child(
                    h_flex().child(
                        Button::new("model-retry")
                            .label("Try again")
                            .on_click(cx.listener(|view, _, _, cx| view.start_download(cx))),
                    ),
                );
            }
            _ => {}
        }

        let ready = self.ready();
        let mut page = v_flex()
            .w_full()
            .max_w(tokens::content_width())
            .gap_4()
            .child(Label::new("Welcome to Isonomic").text_xl().font_semibold())
            .child(
                div()
                    .text_color(muted)
                    .child("A few checks before your first chat. Everything runs on this Mac."),
            )
            .child(
                Stepper::new("first-run-steps")
                    .w_full()
                    .disabled(true)
                    .selected_index(self.current_step().min(2))
                    .items([
                        StepperItem::new().child("Capsule"),
                        StepperItem::new().child("Memory"),
                        StepperItem::new().child("Local model"),
                    ]),
            )
            .child(
                GroupBox::new()
                    .id("first-run-capsule")
                    .title(Label::new("Capsule").font_semibold())
                    .child(line("capsule-ready", capsule)),
            )
            .child(memory)
            .child(model);
        if let Some(error) = &self.error {
            page = page.child(div().id("first-run-failed").test_support().child(
                Alert::error("first-run-failed-alert", error.clone()).title("Couldn't finish"),
            ));
        }
        page = page.child(
            h_flex().justify_end().child(
                Button::new("first-run-continue")
                    .primary()
                    .label(if self.finishing {
                        "Finishing"
                    } else {
                        "Continue"
                    })
                    .disabled(!ready || self.finishing)
                    .on_click(cx.listener(|view, _, _, cx| view.finish(cx))),
            ),
        );
        v_flex()
            .id("first-run")
            .test_support()
            .track_focus(&self.focus)
            .size_full()
            .items_center()
            .p_6()
            .overflow_y_scroll()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(page)
    }
}

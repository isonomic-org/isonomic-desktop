//! First run's layout, after the design's onboarding page (design 0.2,
//! "First run and system states"): on the muted page fill, the wordmark,
//! the stepper, then one card holding the steps, one group per step.
//! Continue stays in a footer pinned at the foot (a recorded deviation: the
//! steps can be taller than the window). Isonomic UI's components as
//! built, its spacing and type, both themes, every action a button.

use ui::base::{h_flex, v_flex};
use ui::component::alert::Alert;
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::clipboard::Clipboard;
use ui::component::group_box::GroupBox;
use ui::component::label::Label;
use ui::component::progress::Progress;
use ui::component::spinner::Spinner;
use ui::component::stepper::{Stepper, StepperItem};
use ui::component::theme::ActiveTheme as _;
use ui::extra::wordmark::Wordmark;
use ui::component::{Disableable as _, Sizable as _, StyledExt as _};
use ui::{
    div, tokens, Context, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, TestSupportExt as _, Window,
};

use super::{FirstRun, IdStep, Memory, ModelStep};

/// A line tests can find by `id`.
fn line(id: &'static str, text: impl Into<SharedString>) -> impl IntoElement {
    div().id(id).test_support().child(text.into())
}

impl FirstRun {
    /// Which step the person is on: the memory check until it passes, then
    /// the model.
    fn current_step(&self) -> usize {
        match (&self.memory, &self.id, &self.model) {
            (Memory::Enough(_), IdStep::Ready(_), ModelStep::Installed(_)) => 4,
            (Memory::Enough(_), IdStep::Ready(_), _) => 3,
            (Memory::Enough(_), _, _) => 2,
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
            "A few checks before your first chat. Your Capsule was created on this Mac: \
             it keeps everything Isonomic stores, and everything runs here."
        } else {
            "A few checks before your first chat. Your Capsule is on this Mac: it keeps \
             everything Isonomic stores, and everything runs here."
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

        let mut id = GroupBox::new()
            .id("first-run-id")
            .title(Label::new("Your Isonomic ID").font_semibold())
            .child(div().text_color(muted).child(
                "Your ID is how your apps, and later your other devices, know it's you. \
                 It's made on this Mac and its key stays here.",
            ));
        match &self.id {
            IdStep::Waiting => {
                id = id.child(line("id-waiting", "Made once the memory check passes."));
            }
            IdStep::Creating => {
                id = id.child(
                    h_flex()
                        .gap_2()
                        .child(Spinner::new().small())
                        .child(line("id-creating", "Creating your ID")),
                );
            }
            IdStep::Ready(value) => {
                id = id.child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .id("id-ready")
                                .test_support()
                                .font_family(theme.mono_font_family.clone())
                                .child(SharedString::from(value.clone())),
                        )
                        .child(
                            Clipboard::new("id-copy")
                                .value(value.clone())
                                .tooltip("Copy your ID")
                                .accessibility_label("Copy your Isonomic ID"),
                        ),
                );
            }
            IdStep::Failed(error) => {
                id = id
                    .child(
                        div().id("id-failed").test_support().child(
                            Alert::error("id-failed-alert", error.clone())
                                .title("Isonomic couldn't create your ID"),
                        ),
                    )
                    .child(
                        h_flex().child(
                            Button::new("id-retry")
                                .label("Try again")
                                .on_click(cx.listener(|view, _, _, cx| view.retry_identity(cx))),
                        ),
                    );
            }
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
                    .child("Chosen once your ID is made."),
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
        // The design's card: white, a 1 px border, 16 radius, 28 padding,
        // 14 between its children, a medium shadow, 600 wide.
        let mut card = v_flex()
            .id("first-run-card")
            .test_support()
            .w_full()
            .max_w(tokens::card_width())
            .p_7()
            .gap_3p5()
            .bg(theme.background)
            .border_1()
            .border_color(theme.border)
            .rounded_2xl()
            .shadow_md()
            .child(
                div()
                    .text_color(muted)
                    .child(line("capsule-ready", capsule)),
            )
            .child(memory)
            .child(id)
            .child(model);
        if let Some(error) = &self.error {
            card = card.child(div().id("first-run-failed").test_support().child(
                Alert::error("first-run-failed-alert", error.clone()).title("Couldn't finish"),
            ));
        }
        // The page: wordmark, stepper, card, 20 apart and centred.
        let page = v_flex()
            .w_full()
            .max_w(tokens::card_width())
            .items_center()
            .gap_4()
            .child(
                // The brand's wordmark: "Isonomic" in the sans, Regular.
                div()
                    .id("first-run-wordmark")
                    .test_support()
                    .child(Wordmark::new().size(tokens::wordmark_size())),
            )
            .child(
                Stepper::new("first-run-steps")
                    .w_full()
                    .disabled(true)
                    .selected_index(self.current_step().min(3))
                    .items([
                        StepperItem::new().child("Capsule"),
                        StepperItem::new().child("Memory"),
                        StepperItem::new().child("Isonomic ID"),
                        StepperItem::new().child("Local model"),
                    ]),
            )
            .child(card);
        let footer = h_flex()
            .id("first-run-footer")
            .test_support()
            .w_full()
            .justify_center()
            .p_4()
            .bg(theme.background)
            .border_t_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .w_full()
                    .max_w(tokens::card_width())
                    .justify_end()
                    .child(
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
        // The steps scroll; Continue stays in view at the foot.
        v_flex()
            .id("first-run")
            .test_support()
            .track_focus(&self.focus)
            .size_full()
            .bg(theme.sidebar)
            .text_color(theme.foreground)
            .child(
                v_flex()
                    .id("first-run-steps-body")
                    .flex_1()
                    .w_full()
                    .items_center()
                    .p_4()
                    .overflow_y_scroll()
                    .child(page),
            )
            .child(footer)
    }
}

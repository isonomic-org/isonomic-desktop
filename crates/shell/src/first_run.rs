//! First run: shown in place of the tabs until it finishes, once per
//! Capsule.
//!
//! Steps: the Capsule (created by the engine link before the window
//! opened), the memory check against V1's floor, and the local model. The
//! model step waits on the SDK: it has no call yet to download a model with
//! progress (isonomic-sdk #21), so the flow says so and lets the user carry
//! on once the memory check passes.
//!
//! Every engine call and the marker write run off the foreground thread.

use std::sync::Arc;

use futures::future::BoxFuture;
use isonomic_client::Client;
use isonomic_proto::HardwareProfile;
use ui::base::{h_flex, v_flex};
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::theme::ActiveTheme as _;
use ui::component::Disableable as _;
use ui::{
    div, AppContext as _, Context, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, TestSupportExt as _, Window,
};

/// V1 runs local models only, and needs this much memory in the machine.
pub const MEMORY_FLOOR_BYTES: u64 = 16 << 30;

/// What first run needs from the engine and the Capsule. The real one is
/// the engine client and the engine link's Capsule; tests script their own.
pub trait Setup: Send + Sync + 'static {
    fn profile(&self) -> BoxFuture<'static, Result<HardwareProfile, String>>;

    /// Remember that first run has finished.
    fn finish(&self) -> BoxFuture<'static, Result<(), String>>;
}

/// The engine's hardware profile, through the shell's client.
pub fn profile_from(client: &Client) -> BoxFuture<'static, Result<HardwareProfile, String>> {
    let client = client.clone();
    Box::pin(async move { client.profile().await.map_err(|e| e.to_string()) })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Memory {
    Checking,
    Enough(HardwareProfile),
    TooLittle(HardwareProfile),
    Failed(String),
}

/// First run has finished; the shell shows its tabs.
pub struct Finished;

pub struct FirstRun {
    setup: Arc<dyn Setup>,
    created: bool,
    memory: Memory,
    finishing: bool,
    error: Option<String>,
}

impl EventEmitter<Finished> for FirstRun {}

fn gib(bytes: u64) -> String {
    format!("{:.0} GB", bytes as f64 / (1u64 << 30) as f64)
}

impl FirstRun {
    /// `created`: the Capsule was made at this launch.
    pub fn new(setup: Arc<dyn Setup>, created: bool, cx: &mut Context<Self>) -> Self {
        let mut first_run = Self {
            setup,
            created,
            memory: Memory::Checking,
            finishing: false,
            error: None,
        };
        first_run.check_memory(cx);
        first_run
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    fn check_memory(&mut self, cx: &mut Context<Self>) {
        self.memory = Memory::Checking;
        cx.notify();
        let asked = self.setup.profile();
        cx.spawn(async move |view, cx| {
            let answer = asked.await;
            let _ = view.update(cx, |view, cx| {
                view.memory = match answer {
                    Ok(p) if p.total_memory_bytes >= MEMORY_FLOOR_BYTES => Memory::Enough(p),
                    Ok(p) => Memory::TooLittle(p),
                    Err(error) => Memory::Failed(error),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        if self.finishing || !matches!(self.memory, Memory::Enough(_)) {
            return;
        }
        self.finishing = true;
        self.error = None;
        cx.notify();
        let written = cx.background_spawn(self.setup.finish());
        cx.spawn(async move |view, cx| {
            let outcome = written.await;
            let _ = view.update(cx, |view, cx| {
                view.finishing = false;
                match outcome {
                    Ok(()) => cx.emit(Finished),
                    Err(error) => view.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn memory_line(&self) -> (&'static str, String) {
        let floor = gib(MEMORY_FLOOR_BYTES);
        match &self.memory {
            Memory::Checking => ("memory-checking", "Checking this Mac's memory".into()),
            Memory::Enough(p) => (
                "memory-enough",
                format!(
                    "{} of memory: enough for local models",
                    gib(p.total_memory_bytes)
                ),
            ),
            Memory::TooLittle(p) => (
                "memory-too-little",
                format!(
                    "{} of memory. Isonomic needs at least {floor} to run its local models.",
                    gib(p.total_memory_bytes)
                ),
            ),
            Memory::Failed(error) => ("memory-failed", format!("Couldn't check memory: {error}")),
        }
    }
}

fn step(id: &'static str, text: impl Into<SharedString>) -> impl IntoElement {
    div().id(id).test_support().child(text.into())
}

impl Render for FirstRun {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let capsule = if self.created {
            "Your Capsule was created on this Mac"
        } else {
            "Your Capsule is on this Mac"
        };
        let (memory_id, memory_text) = self.memory_line();
        let ready = matches!(self.memory, Memory::Enough(_));
        let mut actions = h_flex().gap_2().child(
            Button::new("first-run-continue")
                .primary()
                .label(if self.finishing {
                    "Finishing"
                } else {
                    "Continue"
                })
                .disabled(!ready || self.finishing)
                .on_click(cx.listener(|view, _, _, cx| view.finish(cx))),
        );
        if matches!(self.memory, Memory::Failed(_)) {
            actions = actions.child(
                Button::new("memory-check-again")
                    .label("Check again")
                    .on_click(cx.listener(|view, _, _, cx| view.check_memory(cx))),
            );
        }
        let mut view = v_flex()
            .id("first-run")
            .test_support()
            .size_full()
            .p_6()
            .gap_4()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(div().text_lg().child("Welcome to Isonomic"))
            .child(step("capsule-ready", capsule))
            .child(step(memory_id, memory_text))
            .child(
                div()
                    .id("model-download-unavailable")
                    .test_support()
                    .text_color(theme.muted_foreground)
                    .child("Downloading a local model isn't available in this build yet."),
            )
            .child(actions);
        if let Some(error) = &self.error {
            view = view.child(
                div()
                    .id("first-run-failed")
                    .test_support()
                    .text_color(theme.danger)
                    .child(SharedString::from(format!("Couldn't finish: {error}"))),
            );
        }
        view
    }
}

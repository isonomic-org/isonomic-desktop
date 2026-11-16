//! First run: shown in place of the tabs until it finishes, once per
//! Capsule.
//!
//! Steps: the Capsule (created by the engine link before the window
//! opened), the memory check against V1's floor, then the local model the
//! engine recommends for this machine: downloaded with progress unless it
//! is already installed. Continue needs both the memory and the model.
//!
//! The engine owns the download: closing the window leaves it running, and
//! the next first run attaches to it.
//!
//! Every engine call and the marker write run off the foreground thread.

use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::StreamExt as _;
use isonomic_client::Client;
use isonomic_proto::models::{DownloadOutcome, DownloadProgress, Recommendation};
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

    /// The local model to start with on this machine.
    fn recommended(&self) -> BoxFuture<'static, Result<Recommendation, String>>;

    /// Whether `model`'s file is in place.
    fn installed(&self, model: &str) -> BoxFuture<'static, Result<bool, String>>;

    /// Download `model`, or attach to its running download.
    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>>;

    fn cancel_download(&self, model: &str) -> BoxFuture<'static, Result<(), String>>;

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

/// The local model step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStep {
    /// Until the memory check passes.
    Waiting,
    /// Asking which model, and whether it is installed.
    Choosing,
    /// Not installed; the person starts the download.
    Ready(Recommendation),
    Downloading {
        model: Recommendation,
        received: u64,
        /// `None`: the engine doesn't know the size yet.
        total: Option<u64>,
    },
    Installed(Recommendation),
    Failed {
        model: Option<Recommendation>,
        error: String,
    },
}

/// First run has finished; the shell shows its tabs.
pub struct Finished;

pub struct FirstRun {
    setup: Arc<dyn Setup>,
    created: bool,
    memory: Memory,
    model: ModelStep,
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
            model: ModelStep::Waiting,
            finishing: false,
            error: None,
        };
        first_run.check_memory(cx);
        first_run
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn model(&self) -> &ModelStep {
        &self.model
    }

    /// Which model, and is it already there?
    fn choose_model(&mut self, cx: &mut Context<Self>) {
        self.model = ModelStep::Choosing;
        cx.notify();
        let setup = Arc::clone(&self.setup);
        cx.spawn(async move |view, cx| {
            let step = match setup.recommended().await {
                Err(error) => ModelStep::Failed { model: None, error },
                Ok(model) => match setup.installed(&model.model).await {
                    Ok(true) => ModelStep::Installed(model),
                    Ok(false) => ModelStep::Ready(model),
                    Err(error) => ModelStep::Failed {
                        model: Some(model),
                        error,
                    },
                },
            };
            let _ = view.update(cx, |view, cx| {
                view.model = step;
                cx.notify();
            });
        })
        .detach();
    }

    /// Press Download (or Try again), as the person would.
    pub fn download_now(&mut self, cx: &mut Context<Self>) {
        self.start_download(cx);
    }

    fn start_download(&mut self, cx: &mut Context<Self>) {
        let model = match &self.model {
            ModelStep::Ready(model) => model.clone(),
            ModelStep::Failed {
                model: Some(model), ..
            } => model.clone(),
            ModelStep::Failed { model: None, .. } => return self.choose_model(cx),
            _ => return,
        };
        self.model = ModelStep::Downloading {
            received: 0,
            total: model.size_bytes,
            model: model.clone(),
        };
        cx.notify();
        let mut progress = self.setup.download(&model.model);
        let model_total = model.size_bytes;
        cx.spawn(async move |view, cx| {
            while let Some(event) = progress.next().await {
                let model = model.clone();
                let more = view.update(cx, |view, cx| {
                    let (step, more) = match event {
                        Err(error) => (
                            ModelStep::Failed {
                                model: Some(model),
                                error,
                            },
                            false,
                        ),
                        Ok(p) => match p.outcome {
                            None => (
                                ModelStep::Downloading {
                                    model,
                                    received: p.received_bytes,
                                    total: p.total_bytes.or(model_total),
                                },
                                true,
                            ),
                            Some(DownloadOutcome::Installed) => {
                                (ModelStep::Installed(model), false)
                            }
                            Some(DownloadOutcome::Cancelled) => (ModelStep::Ready(model), false),
                            Some(_) => (
                                ModelStep::Failed {
                                    model: Some(model),
                                    error: p
                                        .error
                                        .map(|e| e.message)
                                        .unwrap_or_else(|| "the download failed".into()),
                                },
                                false,
                            ),
                        },
                    };
                    view.model = step;
                    cx.notify();
                    more
                });
                if !matches!(more, Ok(true)) {
                    return;
                }
            }
            // The stream ended with no outcome: say so, and offer a retry.
            let _ = view.update(cx, |view, cx| {
                if let ModelStep::Downloading { model, .. } = &view.model {
                    view.model = ModelStep::Failed {
                        model: Some(model.clone()),
                        error: "the download stopped".into(),
                    };
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn cancel_download(&mut self, cx: &mut Context<Self>) {
        if let ModelStep::Downloading { model, .. } = &self.model {
            // The stream's last event, `cancelled`, moves the step back.
            cx.background_spawn(self.setup.cancel_download(&model.model))
                .detach();
        }
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
                if matches!(view.memory, Memory::Enough(_)) {
                    view.choose_model(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Press Continue, as the person would: does nothing until the memory
    /// check passed and the model is installed. `true` if it went ahead.
    pub fn continue_if_ready(&mut self, cx: &mut Context<Self>) -> bool {
        let ready = self.ready() && !self.finishing;
        self.finish(cx);
        ready
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        if self.finishing || !self.ready() {
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

    fn ready(&self) -> bool {
        matches!(self.memory, Memory::Enough(_)) && matches!(self.model, ModelStep::Installed(_))
    }

    fn model_line(&self) -> Option<(&'static str, String)> {
        let line = match &self.model {
            ModelStep::Waiting => return None,
            ModelStep::Choosing => ("model-choosing", "Choosing a local model".into()),
            ModelStep::Ready(m) => (
                "model-ready",
                match m.size_bytes {
                    Some(bytes) => format!("{} ({}) will run on this Mac", m.model, size(bytes)),
                    None => format!("{} will run on this Mac", m.model),
                },
            ),
            ModelStep::Downloading {
                model,
                received,
                total,
            } => {
                // Never divide by a size the engine doesn't know.
                let text = match total.filter(|total| *total > 0) {
                    Some(total) => format!(
                        "Downloading {}: {}% ({} of {})",
                        model.model,
                        received.saturating_mul(100) / total,
                        size(*received),
                        size(total)
                    ),
                    None => format!("Downloading {}: {} so far", model.model, size(*received)),
                };
                ("model-downloading", text)
            }
            ModelStep::Installed(m) => ("model-installed", format!("{} is installed", m.model)),
            ModelStep::Failed { error, .. } => {
                ("model-failed", format!("Couldn't get the model: {error}"))
            }
        };
        Some(line)
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

/// A size for people: GB with one decimal, or MB below one GB.
fn size(bytes: u64) -> String {
    let gb = bytes as f64 / (1u64 << 30) as f64;
    if gb >= 1.0 {
        format!("{gb:.1} GB")
    } else {
        format!("{:.0} MB", bytes as f64 / (1u64 << 20) as f64)
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
        let ready = self.ready();
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
        match &self.model {
            ModelStep::Ready(_) => {
                actions = actions.child(
                    Button::new("model-download")
                        .label("Download")
                        .on_click(cx.listener(|view, _, _, cx| view.start_download(cx))),
                )
            }
            ModelStep::Downloading { .. } => {
                actions = actions.child(
                    Button::new("model-cancel")
                        .label("Cancel download")
                        .on_click(cx.listener(|view, _, _, cx| view.cancel_download(cx))),
                )
            }
            ModelStep::Failed { .. } => {
                actions = actions.child(
                    Button::new("model-retry")
                        .label("Try again")
                        .on_click(cx.listener(|view, _, _, cx| view.start_download(cx))),
                )
            }
            _ => {}
        }
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
            .children(self.model_line().map(|(id, text)| step(id, text)))
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

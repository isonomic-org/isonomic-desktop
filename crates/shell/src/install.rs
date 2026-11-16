//! Installing one model an app asked for: the window's side of an app's
//! "this model isn't installed" (the SDK's window request, isonomic-sdk #31).
//!
//! [`Shell::install_model`](crate::Shell::install_model) shows this view in
//! place of the tab on screen and resolves once the person has installed the
//! model, said not now, or the download failed and they gave up. The
//! engine owns the download, as in first run: closing the view leaves it
//! running, and asking again attaches to it.

use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::StreamExt as _;
use isonomic_client::Client;
use isonomic_proto::models::{DownloadOutcome, DownloadProgress};
use ui::base::{h_flex, v_flex};
use ui::component::button::{Button, ButtonVariants as _};
use ui::component::group_box::GroupBox;
use ui::component::label::Label;
use ui::component::progress::Progress;
use ui::component::theme::ActiveTheme as _;
use ui::component::StyledExt as _;
use ui::{
    div, tokens, AppContext as _, Context, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, TestSupportExt as _, Window,
};

use crate::downloads;

/// Where models come from. The real one is the shell's client; tests
/// script their own.
pub trait ModelSource: Send + Sync + 'static {
    fn installed(&self, model: &str) -> BoxFuture<'static, Result<bool, String>>;
    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>>;
    fn cancel_download(&self, model: &str) -> BoxFuture<'static, Result<(), String>>;
}

impl ModelSource for Client {
    fn installed(&self, model: &str) -> BoxFuture<'static, Result<bool, String>> {
        downloads::installed(self, model)
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        downloads::download(self, model)
    }

    fn cancel_download(&self, model: &str) -> BoxFuture<'static, Result<(), String>> {
        downloads::cancel(self, model)
    }
}

/// How an install request ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed,
    /// The person said not now.
    Declined,
    /// The engine couldn't say whether the model is installed.
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStep {
    Checking,
    Ready,
    Downloading { received: u64, total: Option<u64> },
    Failed(String),
}

pub struct ModelInstall {
    source: Arc<dyn ModelSource>,
    model: String,
    /// The app that asked, for the words on screen.
    app: String,
    step: InstallStep,
}

impl EventEmitter<InstallOutcome> for ModelInstall {}

impl ModelInstall {
    pub fn new(
        source: Arc<dyn ModelSource>,
        model: String,
        app: String,
        cx: &mut Context<Self>,
    ) -> Self {
        let asked = source.installed(&model);
        cx.spawn(async move |view, cx| {
            let answer = asked.await;
            let _ = view.update(cx, |view, cx| match answer {
                Ok(true) => cx.emit(InstallOutcome::Installed),
                Ok(false) => {
                    view.step = InstallStep::Ready;
                    cx.notify();
                }
                Err(error) => cx.emit(InstallOutcome::Failed(error)),
            });
        })
        .detach();
        Self {
            source,
            model,
            app,
            step: InstallStep::Checking,
        }
    }

    pub fn step(&self) -> &InstallStep {
        &self.step
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn download(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.step, InstallStep::Ready | InstallStep::Failed(_)) {
            return;
        }
        self.step = InstallStep::Downloading {
            received: 0,
            total: None,
        };
        cx.notify();
        let mut progress = self.source.download(&self.model);
        cx.spawn(async move |view, cx| {
            while let Some(event) = progress.next().await {
                let more = view.update(cx, |view, cx| {
                    let more = match event {
                        Err(error) => {
                            view.step = InstallStep::Failed(error);
                            false
                        }
                        Ok(p) => match p.outcome {
                            None => {
                                view.step = InstallStep::Downloading {
                                    received: p.received_bytes,
                                    total: p.total_bytes,
                                };
                                true
                            }
                            Some(DownloadOutcome::Installed) => {
                                cx.emit(InstallOutcome::Installed);
                                false
                            }
                            Some(DownloadOutcome::Cancelled) => {
                                view.step = InstallStep::Ready;
                                false
                            }
                            Some(_) => {
                                let error = p.error.map(|e| e.message);
                                view.step = InstallStep::Failed(
                                    error.unwrap_or_else(|| "the download failed".into()),
                                );
                                false
                            }
                        },
                    };
                    cx.notify();
                    more
                });
                if !matches!(more, Ok(true)) {
                    return;
                }
            }
            let _ = view.update(cx, |view, cx| {
                if matches!(view.step, InstallStep::Downloading { .. }) {
                    view.step = InstallStep::Failed("the download stopped".into());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        if matches!(self.step, InstallStep::Downloading { .. }) {
            cx.background_spawn(self.source.cancel_download(&self.model))
                .detach();
        }
    }

    fn decline(&mut self, cx: &mut Context<Self>) {
        if matches!(self.step, InstallStep::Downloading { .. }) {
            // The download goes on in the engine; asking again attaches.
            return;
        }
        cx.emit(InstallOutcome::Declined);
    }

    fn line(&self) -> (&'static str, String) {
        let model = &self.model;
        match &self.step {
            InstallStep::Checking => ("install-checking", format!("Checking for {model}")),
            InstallStep::Ready => (
                "install-ready",
                format!(
                    "{} needs the model {model}, which isn't installed yet.",
                    self.app
                ),
            ),
            InstallStep::Downloading { received, total } => {
                let mb = |bytes: u64| format!("{:.0} MB", bytes as f64 / (1u64 << 20) as f64);
                let text = match total.filter(|total| *total > 0) {
                    Some(total) => format!(
                        "Downloading {model}: {}% ({} of {})",
                        received.saturating_mul(100) / total,
                        mb(*received),
                        mb(total)
                    ),
                    None => format!("Downloading {model}: {} so far", mb(*received)),
                };
                ("install-downloading", text)
            }
            // The engine's own words, such as its disk-space refusal.
            InstallStep::Failed(error) => ("install-failed", error.clone()),
        }
    }
}

impl Render for ModelInstall {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (id, text) = self.line();
        let mut actions = h_flex().gap_2();
        match self.step {
            InstallStep::Ready => {
                actions = actions.child(
                    Button::new("install-download")
                        .primary()
                        .label("Download")
                        .on_click(cx.listener(|view, _, _, cx| view.download(cx))),
                )
            }
            InstallStep::Failed(_) => {
                actions = actions.child(
                    Button::new("install-retry")
                        .primary()
                        .label("Try again")
                        .on_click(cx.listener(|view, _, _, cx| view.download(cx))),
                )
            }
            InstallStep::Downloading { .. } => {
                actions = actions.child(
                    Button::new("install-cancel")
                        .label("Cancel download")
                        .on_click(cx.listener(|view, _, _, cx| view.cancel(cx))),
                )
            }
            InstallStep::Checking => {}
        }
        if !matches!(self.step, InstallStep::Downloading { .. }) {
            actions = actions.child(
                Button::new("install-not-now")
                    .label("Not now")
                    .on_click(cx.listener(|view, _, _, cx| view.decline(cx))),
            );
        }
        let mut group = GroupBox::new()
            .id("install-model-group")
            .title(Label::new(self.model.clone()).font_semibold())
            .child(div().id(id).test_support().child(SharedString::from(text)));
        if let InstallStep::Downloading { received, total } = self.step {
            let progress = match total.filter(|total| *total > 0) {
                Some(total) => Progress::new("install-progress")
                    .value((received as f64 / total as f64 * 100.0) as f32),
                None => Progress::new("install-progress").loading(true),
            };
            group = group.child(progress.accessibility_label("Model download"));
        }
        v_flex()
            .id("model-install")
            .test_support()
            .size_full()
            .items_center()
            .p_6()
            .child(
                v_flex()
                    .w_full()
                    .max_w(tokens::content_width())
                    .gap_4()
                    .child(Label::new("Install a model").text_xl().font_semibold())
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("It runs on this Mac and is downloaded once."),
                    )
                    .child(group)
                    .child(actions.justify_end()),
            )
    }
}

/// The window's answer to apps' requests ([`isonomic_app::WindowRequests`]).
/// An app calls it from its own code with no access to the shell, so it
/// queues each request for [`Shell::serve_requests`](crate::Shell::serve_requests).
pub struct ShellRequests {
    queue: futures::channel::mpsc::UnboundedSender<(String, isonomic_app::WindowRequest)>,
}

/// The requests' two ends: hand the first to the app host, the second to
/// the shell once it exists.
pub fn requests() -> (
    Arc<ShellRequests>,
    futures::channel::mpsc::UnboundedReceiver<(String, isonomic_app::WindowRequest)>,
) {
    let (queue, queued) = futures::channel::mpsc::unbounded();
    (Arc::new(ShellRequests { queue }), queued)
}

impl isonomic_app::WindowRequests for ShellRequests {
    fn request(&self, app: &str, request: isonomic_app::WindowRequest) -> bool {
        match request {
            isonomic_app::WindowRequest::InstallModel { .. } => self
                .queue
                .unbounded_send((app.to_string(), request))
                .is_ok(),
            // A kind this window does not handle yet.
            _ => false,
        }
    }
}

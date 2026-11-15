//! The app host: first-party apps are crates compiled into the window.
//!
//! An app crate implements the SDK's `isonomic_app::App` and the window
//! links it with `isonomic_app::register`. Each registration becomes a rail
//! entry. The app is built the first time its tab is opened: the shell asks
//! the engine for the app's token (`apps.grant`, from the app's own
//! manifest), scopes a client to it and hands the app its context. From then
//! on the tab draws the app's root view and the rail follows its attention
//! signal. The shell never calls the engine for the app.

use std::sync::Arc;

use isonomic_app::{AppContext, Attention, LoadedApp, Registration, SettingsStore};
use isonomic_client::Client;
use tokio::sync::watch;
use ui::component::theme::ActiveTheme as _;
use ui::{
    div, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, TestSupportExt as _, Window,
};

/// What the window gives every app: the engine and a settings store.
#[derive(Clone)]
pub struct AppHost {
    client: Client,
    settings: Arc<dyn SettingsStore>,
}

impl AppHost {
    /// `client` is the shell's own client; apps only ever see a client
    /// scoped to their token.
    pub fn new(client: Client, settings: Arc<dyn SettingsStore>) -> Self {
        Self { client, settings }
    }
}

/// Where an app's tab is.
pub enum AppState {
    /// Not opened yet.
    Closed,
    /// Waiting for the engine to grant its token.
    Granting,
    Open(LoadedApp),
    /// The engine refused the grant, or could not be reached.
    Failed(String),
}

/// One app's tab: its registration, its state, and what the rail shows.
pub struct AppTab {
    registration: Registration,
    host: AppHost,
    state: AppState,
    attention: Attention,
}

impl AppTab {
    pub fn new(registration: Registration, host: AppHost) -> Self {
        Self {
            registration,
            host,
            state: AppState::Closed,
            attention: Attention::None,
        }
    }

    pub fn registration(&self) -> &Registration {
        &self.registration
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// What the app last asked of the rail.
    pub fn attention(&self) -> Attention {
        self.attention
    }

    /// The tab was selected. The first time, grant the app's token, then
    /// build the app. Later calls do nothing; a failed grant is tried again.
    pub fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !matches!(self.state, AppState::Closed | AppState::Failed(_)) {
            return;
        }
        self.state = AppState::Granting;
        cx.notify();
        let client = self.host.client.clone();
        let manifest = self.registration.manifest().clone();
        cx.spawn_in(window, async move |tab, cx| {
            let granted = client.grant(manifest).await;
            let _ = tab.update_in(cx, |tab, window, cx| match granted {
                Ok(token) => tab.open(client.scoped(token), window, cx),
                Err(error) => {
                    tab.state = AppState::Failed(error.to_string());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn open(
        &mut self,
        client: isonomic_client::AppClient,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.registration.manifest().id.clone();
        let (context, attention) = AppContext::new(&id, client, Arc::clone(&self.host.settings));
        self.state = AppState::Open(self.registration.open(context, window, cx));
        self.follow(attention, cx);
        cx.notify();
    }

    /// Redraw when the app asks for (or stops asking for) attention.
    fn follow(&mut self, mut attention: watch::Receiver<Attention>, cx: &mut Context<Self>) {
        self.attention = *attention.borrow_and_update();
        cx.spawn(async move |tab, cx| {
            while attention.changed().await.is_ok() {
                let latest = *attention.borrow_and_update();
                let redrawn = tab.update(cx, |tab, cx| {
                    tab.attention = latest;
                    cx.notify();
                });
                if redrawn.is_err() {
                    break; // the tab is gone
                }
            }
        })
        .detach();
    }
}

impl Render for AppTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.registration.manifest().id.clone();
        let host = div()
            .id(SharedString::from(format!("app-{id}")))
            .test_support()
            .size_full();
        let muted = cx.theme().muted_foreground;
        let name = self.registration.manifest().name.clone();
        match &self.state {
            AppState::Open(app) => host.child(app.view()),
            AppState::Closed | AppState::Granting => host.child(
                div()
                    .id("app-opening")
                    .test_support()
                    .p_6()
                    .text_color(muted)
                    .child(format!("Opening {name}")),
            ),
            AppState::Failed(error) => host.child(
                div()
                    .id("app-failed")
                    .test_support()
                    .p_6()
                    .text_color(cx.theme().danger)
                    .child(format!("{name} could not open: {error}")),
            ),
        }
    }
}

/// Make the entity for `registration`'s tab.
pub fn tab(registration: Registration, host: AppHost, cx: &mut ui::App) -> ui::Entity<AppTab> {
    cx.new(|_| AppTab::new(registration, host))
}

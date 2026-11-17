// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The app host: first-party apps are crates compiled into the window.
//!
//! An app crate implements the SDK's `isonomic_app::App` and the window
//! links it with `isonomic_app::register`. Each registration becomes a rail
//! entry. The app is built the first time its tab is opened: the shell asks
//! the engine for the app's token (`apps.grant`, from the app's own
//! manifest), scopes a client to it and hands the app its context. From then
//! on the tab draws the app's root view and the rail follows its attention
//! signal. The shell never calls the engine for the app.
//!
//! The hardware profile caps how many apps stay loaded ([`loaded_cap`]).
//! Past the cap the shell unloads the least recently used: the app's
//! `on_unload` runs, its view is dropped, and the next open builds it
//! again with the token it already has.

use std::sync::Arc;

use isonomic_app::{
    AppContext, Attention, LoadedApp, Registration, SearchHit, SettingsStore, WindowRequests,
};
use isonomic_client::{AppClient, Client};
use tokio::sync::watch;
use ui::base::h_flex;
use ui::component::alert::Alert;
use ui::component::spinner::Spinner;
use ui::component::theme::ActiveTheme as _;
use ui::component::Sizable as _;
use ui::{
    div, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, TestSupportExt as _, Window,
};

/// The rail segment of a deep link to the app with `id`: its id's last
/// part, so `isonomic://chat/...` goes to `com.isonomic.chat`.
pub fn link_segment(id: &str) -> &str {
    id.rsplit('.').next().unwrap_or(id)
}

/// The segment a deep link names, if it is an `isonomic://` link.
pub fn link_target(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("isonomic://")?;
    let segment = rest.split(['/', '?', '#']).next()?;
    (!segment.is_empty()).then_some(segment)
}

/// How many apps stay loaded on a machine of `profile` (the engine
/// supervisor's spec): `None` means no cap.
pub fn loaded_cap(profile: &str) -> Option<usize> {
    match profile {
        "low" => Some(2),
        "high" => None,
        _ => Some(4),
    }
}

/// What the window gives every app: the engine and a settings store.
#[derive(Clone)]
pub struct AppHost {
    client: Client,
    settings: Arc<dyn SettingsStore>,
    /// Where apps' window requests go; none means every request is refused.
    requests: Option<Arc<dyn WindowRequests>>,
}

impl AppHost {
    /// `client` is the shell's own client; apps only ever see a client
    /// scoped to their token.
    pub fn new(client: Client, settings: Arc<dyn SettingsStore>) -> Self {
        Self {
            client,
            settings,
            requests: None,
        }
    }

    /// Hand apps' requests (such as installing a model) to `requests`.
    pub fn with_requests(mut self, requests: Arc<dyn WindowRequests>) -> Self {
        self.requests = Some(requests);
        self
    }
}

/// An app panicked in a call the window made, and a report was written.
pub struct Stopped;

impl ui::EventEmitter<Stopped> for AppTab {}

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
    /// The app's client, once the engine has granted its token. Kept
    /// across an unload.
    client: Option<AppClient>,
    /// Deep links that arrived before the app was built.
    pending_links: Vec<String>,
}

impl AppTab {
    pub fn new(registration: Registration, host: AppHost) -> Self {
        Self {
            registration,
            host,
            state: AppState::Closed,
            attention: Attention::None,
            client: None,
            pending_links: Vec::new(),
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
        if let Some(client) = self.client.clone() {
            self.open(client, window, cx);
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

    /// Hand the app a deep link. An app that isn't built yet gets it once
    /// it is; the caller activates the tab.
    pub fn open_link(&mut self, url: &str, cx: &mut Context<Self>) {
        match &self.state {
            AppState::Open(app) => {
                if let Err(error) = reports::panic::contain(|| app.open_link(url, cx)) {
                    self.stopped(error, cx);
                }
            }
            _ => self.pending_links.push(url.to_string()),
        }
    }

    /// The app panicked in a call the window made: its tab says so, the
    /// panic hook has written an `app_failed` report, and the next open
    /// builds it again.
    fn stopped(&mut self, error: String, cx: &mut Context<Self>) {
        self.state = AppState::Failed(format!("It stopped working ({error})"));
        self.attention = Attention::None;
        cx.emit(Stopped);
        cx.notify();
    }

    /// Ask the app for search hits. An app that isn't built has none.
    pub fn search(&mut self, q: &str, cx: &mut Context<Self>) -> Vec<SearchHit> {
        let found = match &self.state {
            AppState::Open(app) => reports::panic::contain(|| app.search(q, cx)),
            _ => Ok(Vec::new()),
        };
        found.unwrap_or_else(|error| {
            self.stopped(error, cx);
            Vec::new()
        })
    }

    /// Built, or being built: it counts against the cap.
    pub fn is_loaded(&self) -> bool {
        matches!(self.state, AppState::Open(_) | AppState::Granting)
    }

    /// Tell the app, then drop its view. The next activation builds it
    /// again.
    pub fn unload(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.state, AppState::Open(_)) {
            return;
        }
        if let AppState::Open(app) = std::mem::replace(&mut self.state, AppState::Closed) {
            // A panic while unloading is reported; the tab is closed either way.
            let _ = reports::panic::contain(|| app.unload(cx));
        }
        // An unloaded app can't ask for attention.
        self.attention = Attention::None;
        cx.notify();
    }

    fn open(&mut self, client: AppClient, window: &mut Window, cx: &mut Context<Self>) {
        self.client = Some(client.clone());
        let id = self.registration.manifest().id.clone();
        let (mut context, attention) =
            AppContext::new(&id, client, Arc::clone(&self.host.settings));
        if let Some(requests) = &self.host.requests {
            context = context.with_window(Arc::clone(requests));
        }
        let registration = &self.registration;
        let app = match reports::panic::contain(|| registration.open(context, window, cx)) {
            Ok(app) => app,
            Err(error) => return self.stopped(error, cx),
        };
        for url in std::mem::take(&mut self.pending_links) {
            if let Err(error) = reports::panic::contain(|| app.open_link(&url, cx)) {
                return self.stopped(error, cx);
            }
        }
        self.state = AppState::Open(app);
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
                h_flex()
                    .id("app-opening")
                    .test_support()
                    .size_full()
                    .justify_center()
                    .gap_2()
                    .text_color(muted)
                    .child(Spinner::new().small())
                    .child(format!("Opening {name}")),
            ),
            AppState::Failed(error) => host.child(
                div().id("app-failed").test_support().p_6().child(
                    Alert::error(
                        "app-failed-alert",
                        format!("{error}. Open it again from the rail to retry."),
                    )
                    .title(format!("{name} couldn't open")),
                ),
            ),
        }
    }
}

/// Make the entity for `registration`'s tab.
pub fn tab(registration: Registration, host: AppHost, cx: &mut ui::App) -> ui::Entity<AppTab> {
    cx.new(|_| AppTab::new(registration, host))
}

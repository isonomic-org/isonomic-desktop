// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The connectors chosen on "Integrations and connected apps", and their
//! connecting at "Setting up your workspace": each is added with
//! `connections.add`; one that answers that it needs a sign-in has the
//! engine's sign-in page opened in the browser.
//!
//! A token typed into a row goes to the engine as the connection's secret
//! and nowhere else: it is never logged and never kept with the choices.

use std::sync::Arc;

use isonomic_proto::connections::{AddConnection, ConnectionState};
use ui::component::input::{InputEvent, InputState};
use ui::{AppContext as _, Context, Entity, Window};

use super::backend::Backend;
use super::Onboarding;
use crate::connectors::CONNECTORS;

impl Onboarding {
    /// A field for each connector that asks for something typed. Typing
    /// in one switches its connector on; emptying it, off.
    pub(super) fn make_fields(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<(usize, Entity<InputState>)> {
        CONNECTORS
            .iter()
            .enumerate()
            .filter_map(|(index, connector)| Some((index, connector.asks()?)))
            .map(|(index, asks)| {
                let field = cx.new(|cx| {
                    let state = InputState::new(window, cx).placeholder(asks);
                    let token = CONNECTORS[index].access == crate::connectors::Access::Token;
                    state.masked(token)
                });
                cx.subscribe_in(&field, window, move |view, field, event, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        let typed = !field.read(cx).value().trim().is_empty();
                        if let Some(on) = view.links.get_mut(index) {
                            *on = typed;
                        }
                        cx.notify();
                    }
                })
                .detach();
                (index, field)
            })
            .collect()
    }

    /// Connector `index`'s field, if it has one.
    pub(super) fn field(&self, index: usize) -> Option<&Entity<InputState>> {
        self.fields
            .iter()
            .flatten()
            .find(|(at, _)| *at == index)
            .map(|(_, field)| field)
    }

    /// What is typed in connector `index`'s field.
    pub(super) fn typed_for(&self, index: usize, cx: &Context<Self>) -> String {
        self.field(index)
            .map(|field| field.read(cx).value().to_string())
            .unwrap_or_default()
    }

    /// Type something for connector `index`, as a test would.
    pub fn type_for(
        &mut self,
        index: usize,
        typed: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(field) = self.field(index).cloned() {
            field.update(cx, |field, cx| {
                field.set_value(typed.to_owned(), window, cx)
            });
        }
        if let Some(on) = self.links.get_mut(index) {
            *on = !typed.trim().is_empty();
        }
        cx.notify();
    }

    /// May "Connect" on row `index` switch it? One coming soon can't; one
    /// that asks for something typed shows its field first, and is
    /// switched on by typing in it.
    pub(super) fn may_connect(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        let Some(connector) = CONNECTORS.get(index) else {
            return false;
        };
        if !connector.available {
            return false;
        }
        let on = self.links.get(index).copied().unwrap_or(false);
        if connector.asks().is_some() && !on && self.typed_for(index, cx).trim().is_empty() {
            self.asking_for = Some(index);
            cx.notify();
            return false;
        }
        true
    }

    /// The connectors switched on, as the calls that connect them.
    pub(super) fn wanted_connections(&self, cx: &Context<Self>) -> Vec<AddConnection> {
        CONNECTORS
            .iter()
            .enumerate()
            .filter(|(index, _)| self.links.get(*index).copied().unwrap_or(false))
            .filter_map(|(index, connector)| connector.add(&self.typed_for(index, cx)))
            .collect()
    }
}

/// Add each connection; open the sign-in page of each that needs one.
/// What goes wrong is logged by name, never with a secret.
pub(super) async fn connect_all(
    backend: Arc<dyn Backend>,
    wanted: Vec<AddConnection>,
    view: ui::WeakEntity<Onboarding>,
    cx: &mut ui::AsyncApp,
) {
    for add in wanted {
        let name = add.name.clone();
        let connection = match backend.connect(add).await {
            Ok(connection) => connection,
            Err(said) => {
                reports::log::line(&format!(
                    "onboarding: {name} wasn\u{2019}t connected: {said}"
                ));
                continue;
            }
        };
        if connection.state != ConnectionState::NeedsSignIn {
            continue;
        }
        match backend.sign_in(&connection.id).await {
            Ok(page) => {
                let _ = view.update(cx, |_, cx| cx.open_url(&page));
            }
            Err(said) => {
                reports::log::line(&format!(
                    "onboarding: {name}\u{2019}s sign-in didn\u{2019}t start: {said}"
                ));
            }
        }
    }
}

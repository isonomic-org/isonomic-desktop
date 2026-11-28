// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether the person is acting in an app right now, for the user-action
//! token an app needs to decide an approval (`AppContext::user_action`).
//!
//! An app's tab arms the gate from capture-phase listeners on its own
//! root: a mouse press or release, or a key press, that GPUI dispatches
//! through the tab. Pointer events reach the tab only while it is on
//! screen under the pointer, and key events only while focus is inside
//! it, so a background or hidden tab never arms it. The window must also
//! be active. The gate is cleared once that event's dispatch is over, and
//! the first ask takes it, so one input mints at most one token. An app's
//! timer, stream or the model's output never runs inside such a dispatch,
//! so it can't get a token.

use std::sync::{Arc, Mutex};

use isonomic_app::{ShellUserActions, UserActionSource};
use isonomic_client::Agents;
use ui::Window;

/// The app the person is acting in during the current input event, if any.
#[derive(Default)]
pub struct UserInput(Mutex<Option<String>>);

impl UserInput {
    /// An input event for `app` is being dispatched: arm until it ends.
    pub fn arm(self: &Arc<Self>, app: &str, window: &mut Window, cx: &mut ui::App) {
        if !window.is_window_active() {
            return;
        }
        *self.lock() = Some(app.to_string());
        let gate = Arc::clone(self);
        window.defer(cx, move |_, _| gate.clear());
    }

    /// Whether `app` may have a token now. True at most once per input.
    pub fn take(&self, app: &str) -> bool {
        let mut armed = self.lock();
        if armed.as_deref() == Some(app) {
            *armed = None;
            true
        } else {
            false
        }
    }

    pub fn clear(&self) {
        *self.lock() = None;
    }

    /// The window's token source over this gate: the shell's `agents`
    /// mint (`approvals.user_action` with the asking app).
    pub fn source(self: &Arc<Self>, agents: Agents) -> Arc<dyn UserActionSource> {
        let gate = Arc::clone(self);
        Arc::new(ShellUserActions::new(agents, move |app| gate.take(app)))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_input_lets_one_app_ask_once() {
        let gate = UserInput::default();
        *gate.lock() = Some("com.isonomic.chat".into());
        assert!(!gate.take("com.isonomic.other"));
        assert!(gate.take("com.isonomic.chat"));
        assert!(!gate.take("com.isonomic.chat"), "a second ask in one input");
        *gate.lock() = Some("com.isonomic.chat".into());
        gate.clear();
        assert!(!gate.take("com.isonomic.chat"), "after the input ended");
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The full pass's flow: the steps after the prompt run's own, and the
//! relaunch step in the link run.

use std::sync::atomic::Ordering;
use std::time::Instant;

use isonomic_client::Client;
use shell::apps::AppState;
use shell::Shell;
use ui::{AnyWindowHandle, AsyncApp, Entity};

use super::super::say;

/// After the prompt, stop and continue steps: the keys, then, when asked,
/// the full pass. Its steps each print PASS or FAIL; any FAIL fails the run.
pub async fn after_prompt(
    window: AnyWindowHandle,
    shell: &Entity<Shell>,
    chat: &Entity<isonomic_chat::ChatShell>,
    view: &Entity<conversation_ui::ConversationView>,
    client: &Client,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    let keys = super::super::keys::check(window, chat, started, cx).await;
    if !super::asked() {
        return keys.map(|_| ());
    }
    let mut failed = Vec::new();
    let mut note = |n: u8, name: &'static str, outcome: Result<String, String>| {
        if !super::report(started, n, name, outcome) {
            failed.push(name);
        }
    };
    note(
        4,
        "try-again",
        super::try_again(window, view, started, cx).await,
    );
    note(5, "edit-prompt", super::edit(window, view, cx).await);
    note(
        6,
        "chat-list",
        super::chat_list(window, chat, started, cx).await,
    );
    note(7, "search", super::search(window, chat, started, cx).await);
    let keys = keys.and_then(|k| {
        (k.sidebar_toggled && k.sidebar_back && k.next_chat_opened)
            .then(|| "both sidebar keys and the next-chat key".to_string())
            .ok_or_else(|| format!("{k:?}"))
    });
    note(8, "keys", keys);
    note(
        10,
        "model-picker",
        super::more::picker(window, chat, started, cx).await,
    );
    note(
        11,
        "acknowledgements",
        super::more::acknowledgements(window, started, cx).await,
    );
    if let Some(capsule) = super::super::capsule() {
        note(
            12,
            "problem-report-notice",
            super::more::notice_screen(window, shell, &capsule, started, cx).await,
        );
    }
    note(13, "stopped-thread-drawn", super::drawn::outcome());
    match super::more::state(chat, client, cx).await {
        Ok(state) => say(started, &format!("state {}", state.line())),
        Err(why) => failed.push(Box::leak(format!("state: {why}").into_boxed_str())),
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("full pass: {} failed", failed.join(", ")))
    }
}

/// Step 9, in the link run: the relaunch found what the prompt run left.
pub async fn relaunch_step(
    window: AnyWindowHandle,
    tab: &Entity<shell::apps::AppTab>,
    client: &Client,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    let chat = cx
        .update(|cx| match tab.read(cx).state() {
            AppState::Open(app) => app
                .view()
                .downcast::<isonomic_chat::Chat>()
                .ok()
                .map(|c| c.read(cx).shell().clone()),
            _ => None,
        })
        .ok_or("Chat's root view isn't Chat")?;
    let outcome = match std::env::var(super::more::ENV_EXPECT)
        .ok()
        .and_then(|line| super::more::State::parse(&line))
    {
        Some(expected) => {
            let shown = super::super::FIRST_RUN_SHOWN.load(Ordering::Relaxed);
            super::more::relaunch(&expected, shown, window, &chat, client, cx).await
        }
        None => Err(format!(
            "{} is not set: the prompt run's state is missing",
            super::more::ENV_EXPECT
        )),
    };
    if super::report(started, 9, "relaunch", outcome) {
        Ok(())
    } else {
        Err("full pass: relaunch failed".into())
    }
}

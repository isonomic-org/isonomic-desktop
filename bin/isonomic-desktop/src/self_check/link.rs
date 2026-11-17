// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! `ISONOMIC_SELF_CHECK=link`: the `isonomic://` link given at launch opens
//! its app, and Chat receives it. In the full pass this is also the
//! relaunch on the prompt run's Capsule (step 9).

use std::time::Instant;

use isonomic_client::Client;
use shell::apps::{link_segment, link_target, AppState};
use shell::Shell;
use ui::{AnyWindowHandle, AsyncApp, Entity};

use super::{ready, say, until, CHAT};

pub(super) async fn check(
    links: Vec<String>,
    window: AnyWindowHandle,
    shell: &Entity<Shell>,
    client: &Client,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    let link = links
        .into_iter()
        .next()
        .ok_or("no isonomic:// link was given as a launch argument")?;
    let target = link_target(&link)
        .ok_or("not an isonomic:// link")?
        .to_string();
    ready(window, shell, client, started, cx).await?;
    let handled = window
        .update(cx, |_, window, cx| {
            shell.update(cx, |s, cx| s.open_link(&link, window, cx))
        })
        .map_err(|e| e.to_string())?;
    if !handled {
        return Err(format!("no mounted app is named {target:?}"));
    }
    say(started, &format!("opened {link}"));
    let id = cx.update(|cx| shell.read(cx).selected_tab().to_string());
    if link_segment(&id) != target {
        return Err(format!("the tab on screen is {id}, not the link's app"));
    }
    let tab = cx
        .update(|cx| shell.read(cx).app(&id).cloned())
        .ok_or("no tab")?;
    // Open, or failed with the engine's reason: a failure says why at
    // once instead of waiting out the step.
    until(cx, &tab, "the app to open", |tab, _| {
        matches!(tab.state(), AppState::Open(_) | AppState::Failed(_))
    })
    .await?;
    let failed = cx.update(|cx| match tab.read(cx).state() {
        AppState::Failed(error) => Some(error.clone()),
        _ => None,
    });
    if let Some(error) = failed {
        return Err(format!("{id} could not open: {error}"));
    }
    say(started, &format!("{id} on screen"));
    if id == CHAT {
        let got = cx.update(|cx| match tab.read(cx).state() {
            AppState::Open(app) => app
                .view()
                .downcast::<isonomic_chat::Chat>()
                .ok()
                .map(|chat| chat.read(cx).opened().to_vec()),
            _ => None,
        });
        if !got.unwrap_or_default().contains(&link) {
            return Err("Chat didn't receive the link".into());
        }
        say(started, "Chat received the link");
    }
    if super::full::asked() {
        return super::full::flow::relaunch_step(window, &tab, client, started, cx).await;
    }
    Ok(())
}

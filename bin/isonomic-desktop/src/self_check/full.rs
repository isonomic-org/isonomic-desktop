// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The full self-check (`ISONOMIC_SELF_CHECK_FULL=1`): after the prompt,
//! stop and continue steps, everything Omar would otherwise try by hand,
//! driven through the window's own dispatch and Chat's own entry points,
//! never the system's mouse or keyboard. One line per step, PASS or FAIL
//! with its reason; a FAIL does not stop the steps after it.
//!
//! Steps here: 4 Try again, 5 edit a prompt, 6 the chat list, 7 search.
//! The rest are in [`more`].

pub mod drawn;
pub mod flow;
#[cfg(test)]
mod headless;
#[cfg(test)]
mod headless_stop;
pub mod more;

use std::time::{Duration, Instant};

use conversation_ui::ConversationView;
use isonomic_chat::{ChatShell, Route};
use ui::{AnyWindowHandle, AsyncApp, Entity, Keystroke, SharedString};

use super::screens;
use super::{say, until_within};

/// The environment variable that asks for the full pass.
pub const ENV: &str = "ISONOMIC_SELF_CHECK_FULL";
/// How long a step that waits for the model may take.
const MODEL_STEP: Duration = Duration::from_secs(300);
/// How long a step that waits only for the window may take.
const UI_STEP: Duration = Duration::from_secs(10);
/// A prompt for the extra chats: short, so they end quickly.
const SHORT_PROMPT: &str = "Say hello in three words";
/// The new title the rename types.
pub const RENAMED: &str = "Renamed by the self-check";

pub fn asked() -> bool {
    std::env::var(ENV).is_ok_and(|v| v == "1")
}

/// Print one step's line. `true` when it passed.
pub fn report(started: Instant, n: u8, name: &str, outcome: Result<String, String>) -> bool {
    let (word, why, passed) = match outcome {
        Ok(why) => ("PASS", why, true),
        Err(why) => ("FAIL", why, false),
    };
    say(started, &format!("full {n:02} {name}: {word} ({why})"));
    passed
}

/// A screenshot, saying so only when it couldn't be taken.
pub fn shot(
    window: AnyWindowHandle,
    name: &str,
    dark_too: bool,
    started: Instant,
    cx: &mut AsyncApp,
) {
    let taken = cx.update(|cx| {
        if dark_too {
            screens::both(window, name, cx)
        } else {
            screens::shot(window, name, cx)
        }
    });
    if let Err(why) = taken {
        say(started, &format!("screen {name}: {why}"));
    }
}

/// Each turn's reply as the thread holds it now, in one line: its length
/// and first words. Printed beside the stopped-state screenshot, so a
/// reply drawn short can be told from a reply held short.
pub fn describe_turns(view: &Entity<ConversationView>, cx: &mut AsyncApp) -> String {
    let turns: Vec<String> = cx.update(|cx| {
        view.read(cx)
            .thread()
            .turns()
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let reply = t.reply();
                let start: String = reply.chars().take(40).collect();
                format!(
                    "turn {}: {} characters, {start:?}",
                    i + 1,
                    reply.chars().count()
                )
            })
            .collect()
    });
    format!("at the stop the thread holds: {}", turns.join("; "))
}

/// A screenshot of first run in progress, in the full pass: as it first
/// appears, or with a download running once one has started.
pub async fn first_run_shot(
    window: AnyWindowHandle,
    first_run: &Entity<shell::first_run::FirstRun>,
    name: &str,
    started: Instant,
    cx: &mut AsyncApp,
) {
    if !asked() {
        return;
    }
    if name.ends_with("downloading") {
        let _ = until_within(cx, first_run, "the download to show", UI_STEP, |run, _| {
            matches!(run.model(), shell::first_run::ModelStep::Downloading { .. })
        })
        .await;
    }
    shot(window, name, true, started, cx);
}

pub fn press(window: AnyWindowHandle, key: &'static str, cx: &mut AsyncApp) -> Result<(), String> {
    let keystroke = Keystroke::parse(key).map_err(|_| format!("{key} is not a key"))?;
    window
        .update(cx, |_, window, cx| {
            window.dispatch_keystroke(keystroke, cx);
        })
        .map_err(|e| e.to_string())
}

/// Type `text` into whatever has focus, one key at a time, replacing what
/// is selected.
pub fn type_text(window: AnyWindowHandle, text: &str, cx: &mut AsyncApp) -> Result<(), String> {
    for c in text.chars() {
        let key = if c == ' ' {
            "space".to_string()
        } else {
            c.to_lowercase().to_string()
        };
        let keystroke = Keystroke {
            modifiers: ui::Modifiers {
                shift: c.is_uppercase(),
                ..Default::default()
            },
            key,
            key_char: Some(c.to_string()),
        };
        window
            .update(cx, |_, window, cx| {
                window.dispatch_keystroke(keystroke, cx);
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
const SELECT_ALL: &str = "cmd-a";
#[cfg(not(target_os = "macos"))]
const SELECT_ALL: &str = "ctrl-a";

fn idle(view: &ConversationView) -> bool {
    !view.input_state().is_busy()
}

/// Step 4: Try again under turn one's reply: a second version, and the switcher
/// reads 2 / 2, then 1 / 2, then 2 / 2.
pub async fn try_again(
    window: AnyWindowHandle,
    view: &Entity<ConversationView>,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    until_within(cx, view, "Try again to be offered", UI_STEP, |v, _| {
        v.can_regenerate(0)
    })
    .await?;
    let _ = window.update(cx, |_, _, cx| view.update(cx, |v, cx| v.regenerate(0, cx)));
    until_within(cx, view, "the second version to end", MODEL_STEP, |v, _| {
        idle(v) && v.versions(0) == Some((2, 1))
    })
    .await?;
    shot(window, "thread-finished", true, started, cx);
    let flip = |to: usize, cx: &mut AsyncApp| {
        let _ = window.update(cx, |_, _, cx| {
            view.update(cx, |v, cx| v.show_version(0, to, cx))
        });
    };
    flip(0, cx);
    until_within(cx, view, "the switcher to read 1 / 2", UI_STEP, |v, _| {
        v.versions(0) == Some((2, 0))
    })
    .await?;
    flip(1, cx);
    until_within(cx, view, "the switcher to read 2 / 2", UI_STEP, |v, _| {
        v.versions(0) == Some((2, 1))
    })
    .await?;
    Ok("a second version; the switcher read 2 / 2, 1 / 2, 2 / 2".into())
}

/// Step 5: Edit turn one's prompt and save: a third version from that point,
/// reachable through the switcher.
pub async fn edit(
    window: AnyWindowHandle,
    view: &Entity<ConversationView>,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    until_within(cx, view, "the prompt to be editable", UI_STEP, |v, _| {
        v.can_regenerate(0)
    })
    .await?;
    window
        .update(cx, |_, window, cx| {
            view.update(cx, |v, cx| v.begin_edit(0, window, cx))
        })
        .map_err(|e| e.to_string())?;
    until_within(cx, view, "the edit field", UI_STEP, |v, _| {
        v.editing() == Some(0)
    })
    .await?;
    press(window, SELECT_ALL, cx)?;
    type_text(window, "Plan one evening in Porto", cx)?;
    window
        .update(cx, |_, window, cx| {
            view.update(cx, |v, cx| v.save_edit(window, cx))
        })
        .map_err(|e| e.to_string())?;
    until_within(cx, view, "the edited version to end", MODEL_STEP, |v, _| {
        idle(v) && v.versions(0) == Some((3, 2))
    })
    .await?;
    let prompt = cx.update(|cx| {
        view.read(cx)
            .thread()
            .turns()
            .first()
            .map(|t| t.prompt.clone())
            .unwrap_or_default()
    });
    if !prompt.contains("Porto") {
        return Err(format!("the edited version's prompt is {prompt:?}"));
    }
    let _ = window.update(cx, |_, _, cx| {
        view.update(cx, |v, cx| v.show_version(0, 0, cx))
    });
    until_within(
        cx,
        view,
        "the first version through the switcher",
        UI_STEP,
        |v, _| v.versions(0) == Some((3, 0)),
    )
    .await?;
    let _ = window.update(cx, |_, _, cx| {
        view.update(cx, |v, cx| v.show_version(0, 2, cx))
    });
    until_within(cx, view, "back to the edited version", UI_STEP, |v, _| {
        v.versions(0) == Some((3, 2))
    })
    .await?;
    Ok("the edit made version 3 of 3, and 1 / 3 and 3 / 3 are reachable".into())
}

/// Start a chat with a short prompt, as from home, and wait for its reply.
async fn new_chat(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    cx: &mut AsyncApp,
) -> Result<SharedString, String> {
    new_chat_with(window, chat, SHORT_PROMPT, cx).await
}

/// Start a chat with `prompt`, as from home, and wait for its reply.
pub(super) async fn new_chat_with(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    prompt: &str,
    cx: &mut AsyncApp,
) -> Result<SharedString, String> {
    let before: Vec<SharedString> = cx.update(|cx| {
        chat.read(cx)
            .sessions()
            .iter()
            .map(|s| s.id.clone())
            .collect()
    });
    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| {
                s.new_chat(window, cx);
                s.set_draft(prompt, window, cx);
            });
        })
        .map_err(|e| e.to_string())?;
    press(window, "enter", cx)?;
    until_within(
        cx,
        chat,
        "the new chat",
        UI_STEP,
        |s, _| matches!(s.route(), Route::Session(id) if !before.contains(id)),
    )
    .await?;
    let id = cx.update(|cx| match chat.read(cx).route() {
        Route::Session(id) => id.clone(),
        Route::Home => SharedString::default(),
    });
    let view = cx
        .update(|cx| chat.read(cx).thread(&id).cloned())
        .ok_or("the new chat has no thread")?;
    until_within(cx, &view, "the new chat's reply", MODEL_STEP, |v, _| {
        idle(v)
            && v.thread()
                .turns()
                .last()
                .is_some_and(|t| !t.state.is_open())
    })
    .await?;
    // A new chat is known by a provisional id until the engine names its
    // session; wait for the chat list to hold the route's id.
    until_within(cx, chat, "the chat's id from the engine", UI_STEP, |s, _| {
        matches!(s.route(), Route::Session(now) if s.sessions().iter().any(|x| &x.id == now))
    })
    .await?;
    let named = cx.update(|cx| match chat.read(cx).route() {
        Route::Session(now) => now.clone(),
        Route::Home => id,
    });
    Ok(named)
}

/// Step 6: Three chats; through each row's menu, pin one (it moves to the
/// top), rename one (the name holds after the list is read again from
/// Chat's store), delete one (gone).
pub async fn chat_list(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    let second = new_chat(window, chat, cx).await?;
    let third = new_chat(window, chat, cx).await?;
    let count = cx.update(|cx| chat.read(cx).sessions().len());
    if count != 3 {
        return Err(format!("{count} chats, not 3"));
    }
    menu(window, &third, "pin", cx)?;
    until_within(cx, chat, "the pinned chat at the top", UI_STEP, |s, _| {
        s.sessions()
            .first()
            .is_some_and(|f| f.id == third && f.pinned)
    })
    .await?;
    shot(window, "sidebar-pinned", true, started, cx);

    menu(window, &second, "rename", cx)?;
    until_within(cx, chat, "the rename field", UI_STEP, |s, _| {
        s.is_renaming()
    })
    .await?;
    press(window, SELECT_ALL, cx)?;
    type_text(window, RENAMED, cx)?;
    press(window, "enter", cx)?;
    until_within(cx, chat, "the new name", UI_STEP, |s, _| {
        !s.is_renaming()
            && s.sessions()
                .iter()
                .any(|x| x.id == second && x.title.as_ref() == RENAMED)
    })
    .await?;
    // `is_loaded` stays true after the first load: wait for this reload to
    // finish by Chat's count of finished loads.
    let before = cx.update(|cx| chat.read(cx).loads_finished());
    let _ = window.update(cx, |_, _, cx| chat.update(cx, |s, cx| s.reload(cx)));
    until_within(
        cx,
        chat,
        "the list read again with the name",
        MODEL_STEP,
        |s, _| {
            s.loads_finished() > before
                && s.sessions()
                    .iter()
                    .any(|x| x.id == second && x.title.as_ref() == RENAMED)
        },
    )
    .await?;

    menu(window, &third, "delete", cx)?;
    until_within(cx, chat, "the deleted chat gone", UI_STEP, |s, _| {
        !s.sessions().iter().any(|x| x.id == third)
    })
    .await?;
    Ok("3 chats; through the row menu: pinned to the top, renamed (it held after a reload), deleted (gone)".into())
}

/// Open the row menu of chat `id` and choose `item` (`pin`, `rename`, …).
fn menu(window: AnyWindowHandle, id: &str, item: &str, cx: &mut AsyncApp) -> Result<(), String> {
    click(window, format!("chat-row-menu-{id}"), cx)?;
    click(window, format!("chat-row-menu-{id}-{item}"), cx)
}

/// Click the element `id` through the window's own events, once it is on
/// screen. Needs Isonomic UI's test support: the check build, or a test.
#[cfg(any(test, feature = "check-build"))]
pub fn click(
    window: AnyWindowHandle,
    id: impl Into<ui::ElementId>,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    use ui::test::TestWindowExt as _;
    let id = id.into();
    window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            if window.try_find(id.clone()).is_none() {
                return Err(format!("no {id:?} on screen"));
            }
            window.click(id, cx);
            window.render_frame(cx);
            Ok(())
        })
        .map_err(|e| e.to_string())?
}

#[cfg(not(any(test, feature = "check-build")))]
pub fn click(
    _: AnyWindowHandle,
    id: impl Into<ui::ElementId>,
    _: &mut AsyncApp,
) -> Result<(), String> {
    Err(format!("{:?}: {}", id.into(), screens::NOT_THIS_BUILD))
}

/// An element's accessible label, if it is on screen.
#[cfg(any(test, feature = "check-build"))]
pub fn label(
    window: AnyWindowHandle,
    id: impl Into<ui::ElementId>,
    cx: &mut AsyncApp,
) -> Option<String> {
    use ui::test::TestWindowExt as _;
    let id = id.into();
    window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            window
                .try_find(id)
                .and_then(|e| e.label().map(str::to_string))
        })
        .ok()
        .flatten()
}

#[cfg(not(any(test, feature = "check-build")))]
pub fn label(_: AnyWindowHandle, _: impl Into<ui::ElementId>, _: &mut AsyncApp) -> Option<String> {
    None
}

/// Step 7: Search: the palette opens, a word from the renamed title is
/// typed, its row shows with its last-active text, and Return opens that
/// chat.
pub async fn search(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    let renamed = cx
        .update(|cx| {
            chat.read(cx)
                .sessions()
                .iter()
                .find(|s| s.title.as_ref() == RENAMED)
                .map(|s| s.id.clone())
        })
        .ok_or("no renamed chat to search for")?;
    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| {
                s.new_chat(window, cx);
                s.open_search(window, cx);
            })
        })
        .map_err(|e| e.to_string())?;
    until_within(cx, chat, "the search palette", UI_STEP, |s, _| {
        s.is_searching()
    })
    .await?;
    type_text(window, "Renamed", cx)?;
    let active = label(window, format!("command-{renamed}-meta"), cx)
        .ok_or("the renamed chat's row shows no last-active text")?;
    shot(window, "search", true, started, cx);
    press(window, "enter", cx)?;
    until_within(cx, chat, "the found chat to open", UI_STEP, |s, _| {
        !s.is_searching() && *s.route() == Route::Session(renamed.clone())
    })
    .await?;
    Ok(format!(
        "typed \"Renamed\"; its row read \"{active}\"; Return opened it"
    ))
}

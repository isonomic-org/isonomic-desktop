//! The full self-check (`ISONOMIC_SELF_CHECK_FULL=1`): after the prompt,
//! stop and continue steps, everything Omar would otherwise try by hand,
//! driven through the window's own dispatch and Chat's own entry points,
//! never the system's mouse or keyboard. One line per step, PASS or FAIL
//! with its reason; a FAIL does not stop the steps after it.
//!
//! Steps here: 4 Try again, 5 edit a prompt, 6 the chat list, 7 search.
//! The rest are in [`more`].

pub mod flow;
#[cfg(test)]
mod headless;
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
                s.set_draft(SHORT_PROMPT, window, cx);
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

/// Step 6: Three chats; pin one (it moves to the top), rename one (the name
/// holds after the list is read again), delete one (gone). Pin, rename and
/// delete go through Chat's shell: its rows have no test ids yet.
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
    let _ = window.update(cx, |_, _, cx| {
        chat.update(cx, |s, cx| s.set_pinned(&third, true, cx))
    });
    until_within(cx, chat, "the pinned chat at the top", UI_STEP, |s, _| {
        s.sessions()
            .first()
            .is_some_and(|f| f.id == third && f.pinned)
    })
    .await?;
    shot(window, "sidebar-pinned", true, started, cx);

    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| {
                s.open_session(second.clone(), window, cx);
                s.begin_rename(window, cx);
            })
        })
        .map_err(|e| e.to_string())?;
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
    let _ = window.update(cx, |_, _, cx| chat.update(cx, |s, cx| s.reload(cx)));
    until_within(
        cx,
        chat,
        "the list read again with the name",
        MODEL_STEP,
        |s, _| {
            s.is_loaded()
                && s.sessions()
                    .iter()
                    .any(|x| x.id == second && x.title.as_ref() == RENAMED)
        },
    )
    .await?;

    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| s.delete_session(&third, window, cx))
        })
        .map_err(|e| e.to_string())?;
    until_within(cx, chat, "the deleted chat gone", UI_STEP, |s, _| {
        !s.sessions().iter().any(|x| x.id == third)
    })
    .await?;
    Ok("3 chats; the pinned one moved to the top; the rename held after a reload; the deleted one is gone (through Chat's shell, not the row menu)".into())
}

/// Step 7: Search: the palette opens, a word from the renamed title is typed,
/// and Return opens that chat. The row's last-active text is in the
/// screenshot; the palette's rows have no test ids yet.
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
    shot(window, "search", true, started, cx);
    press(window, "enter", cx)?;
    until_within(cx, chat, "the found chat to open", UI_STEP, |s, _| {
        !s.is_searching() && *s.route() == Route::Session(renamed.clone())
    })
    .await?;
    Ok("typed \"Renamed\"; Return opened the renamed chat".into())
}

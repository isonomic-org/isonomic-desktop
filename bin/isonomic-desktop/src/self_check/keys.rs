//! Chat's keyboard shortcuts, through the window's own key dispatch, after
//! the prompt check: both sidebar toggles, and the next chat. Each result is
//! printed as `yes` or `no`; neither changes PASS or FAIL.

use std::time::{Duration, Instant};

use isonomic_chat::{ChatShell, Route};
use ui::{AnyWindowHandle, AsyncApp, Entity, Keystroke};

use super::{say, until_within};

/// How long a shortcut has to show its effect.
const KEY_LIMIT: Duration = Duration::from_secs(5);

/// Chat's shortcuts: the design's sidebar toggle, Chat's second one (on
/// macOS Cmd-. also means cancel), a new chat, the chat below. Chat binds
/// them in its own key context; the window binds only Quit and Hide.
#[cfg(target_os = "macos")]
const KEYS: [&str; 4] = ["cmd-.", "cmd-b", "cmd-shift-o", "alt-down"];
#[cfg(not(target_os = "macos"))]
const KEYS: [&str; 4] = ["ctrl-.", "ctrl-b", "ctrl-shift-o", "alt-down"];

fn press(window: AnyWindowHandle, key: &'static str, cx: &mut AsyncApp) -> Result<(), String> {
    let keystroke = Keystroke::parse(key).map_err(|e| format!("{key}: {e}"))?;
    window
        .update(cx, |_, window, cx| {
            window.dispatch_keystroke(keystroke, cx);
        })
        .map_err(|e| e.to_string())
}

/// What the keys did.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Keys {
    pub sidebar_toggled: bool,
    /// The second toggle put it back.
    pub sidebar_back: bool,
    pub next_chat_opened: bool,
}

/// Send the keys and print what they did.
pub(super) async fn check(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<Keys, String> {
    let [sidebar, sidebar_too, new_chat, next] = KEYS;

    // The sidebar toggle, then the second toggle to put it back.
    let open = cx.update(|cx| chat.read(cx).sidebar_open());
    press(window, sidebar, cx)?;
    let toggled = until_within(cx, chat, "the sidebar to toggle", KEY_LIMIT, |s, _| {
        s.sidebar_open() != open
    })
    .await
    .is_ok();
    say(
        started,
        &format!("keys: {sidebar} toggled the sidebar: {}", yes(toggled)),
    );
    press(window, sidebar_too, cx)?;
    let back = until_within(cx, chat, "the sidebar to toggle back", KEY_LIMIT, |s, _| {
        s.sidebar_open() == open
    })
    .await
    .is_ok()
        && toggled;
    say(
        started,
        &format!(
            "keys: {sidebar_too} toggled the sidebar back: {}",
            yes(back)
        ),
    );

    // From a new chat, the chat below is the first in the sidebar.
    let first = cx.update(|cx| chat.read(cx).sessions().first().map(|s| s.id.clone()));
    press(window, new_chat, cx)?;
    let _ = until_within(cx, chat, "a new chat", KEY_LIMIT, |s, _| {
        matches!(s.route(), Route::Home)
    })
    .await;
    press(window, next, cx)?;
    let opened = match first {
        Some(first) => until_within(cx, chat, "the next chat", KEY_LIMIT, |s, _| {
            *s.route() == Route::Session(first.clone())
        })
        .await
        .is_ok(),
        None => false,
    };
    say(
        started,
        &format!("keys: {next} opened the next chat: {}", yes(opened)),
    );
    Ok(Keys {
        sidebar_toggled: toggled,
        sidebar_back: back,
        next_chat_opened: opened,
    })
}

fn yes(done: bool) -> &'static str {
    if done {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use ui::{AppContext as _, Focusable as _, TestAppContext, WindowOptions};

    #[ui::test]
    fn the_shortcuts_reach_chat_through_the_windows_dispatch(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ui::init(cx);
            isonomic_chat::init(cx);
        });
        let (window, chat) = cx.update(|cx| {
            ui::open_window(WindowOptions::default(), cx, |window, cx| {
                let chat = cx.new(|cx| ChatShell::new(None, window, cx));
                let focus = chat.read(cx).focus_handle(cx);
                window.focus(&focus, cx);
                chat
            })
            .expect("open the test window")
        });
        cx.run_until_parked();
        assert!(cx.update(|cx| !chat.read(cx).sessions().is_empty()));
        let found = Rc::new(RefCell::new(None));
        let into = Rc::clone(&found);
        cx.spawn(|mut cx| async move {
            *into.borrow_mut() = Some(check(window, &chat, Instant::now(), &mut cx).await);
        })
        .detach();
        cx.run_until_parked();
        let keys = found.borrow_mut().take().expect("the check finished");
        assert_eq!(
            keys.unwrap(),
            Keys {
                sidebar_toggled: true,
                sidebar_back: true,
                next_chat_opened: true,
            }
        );
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Step 13: After the second prompt is stopped, the first reply is drawn
//! whole. Its drawn text is read from the last painted frame, inside the
//! reply's element, and compared with the reply the thread holds.

use std::sync::Mutex;

use conversation_ui::ConversationView;
use ui::{AnyWindowHandle, AsyncApp, Entity};

/// The step's outcome, taken at the stop and reported with the later steps.
static AT_STOP: Mutex<Option<Result<String, String>>> = Mutex::new(None);

/// Check the first reply now, and keep the outcome for step 13.
pub fn check_at_stop(window: AnyWindowHandle, view: &Entity<ConversationView>, cx: &mut AsyncApp) {
    let outcome = first_reply_whole(window, view, cx);
    *AT_STOP.lock().unwrap_or_else(|e| e.into_inner()) = Some(outcome);
}

/// The outcome kept at the stop, for step 13.
pub fn outcome() -> Result<String, String> {
    AT_STOP
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .unwrap_or_else(|| Err("the stop was not reached".into()))
}

/// Letters and digits only: markup is drawn as styling, not characters.
fn letters(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphanumeric()).count()
}

/// Whether `drawn` letters cover the reply's `held`, allowing for markup
/// that draws fewer characters (a link's address, a code fence's language).
pub fn covers(drawn: usize, held: usize) -> bool {
    held > 0 && drawn * 4 >= held * 3
}

/// The first reply as drawn against the first reply as held.
pub fn first_reply_whole(
    window: AnyWindowHandle,
    view: &Entity<ConversationView>,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    let held = cx.update(|cx| {
        view.read(cx)
            .thread()
            .turns()
            .first()
            .map(|t| t.reply())
            .unwrap_or_default()
    });
    let held = letters(&held);
    if held == 0 {
        return Err("the first turn holds no reply".into());
    }
    let drawn = drawn_text(window, 0, cx)?;
    let count = letters(&drawn);
    let start: String = drawn.chars().take(40).collect();
    let line = format!("turn 1 drawn with {count} of its {held} letters and digits, {start:?}");
    if covers(count, held) {
        Ok(line)
    } else {
        Err(line)
    }
}

/// The text painted inside reply `index`'s element in a fresh frame.
#[cfg(any(test, feature = "check-build"))]
pub fn drawn_text(
    window: AnyWindowHandle,
    index: u64,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    use ui::test::TestWindowExt as _;
    let reply = ui::ElementId::NamedInteger("reply".into(), index);
    window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            // Only painted text is recorded: a reply scrolled out of the
            // list's view has none, and says so.
            let texts: Vec<String> = ui::base::test_support::texts(window)
                .iter()
                .filter(|t| t.path().contains(&reply))
                .map(|t| t.text().to_string())
                .collect();
            if texts.is_empty() {
                return Err(format!("reply {} has no drawn text", index + 1));
            }
            Ok(texts.join(" "))
        })
        .map_err(|e| e.to_string())?
}

#[cfg(not(any(test, feature = "check-build")))]
pub fn drawn_text(_: AnyWindowHandle, _: u64, _: &mut AsyncApp) -> Result<String, String> {
    Err(super::super::screens::NOT_THIS_BUILD.to_string())
}

#[cfg(test)]
mod tests {
    use super::covers;

    #[test]
    fn a_reply_drawn_as_its_first_word_does_not_cover() {
        assert!(covers(1000, 1000));
        assert!(covers(800, 1000));
        assert!(!covers(4, 1000));
        assert!(!covers(0, 0));
    }
}

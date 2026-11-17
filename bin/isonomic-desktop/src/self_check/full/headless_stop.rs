// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a thread holds after the self-check's stop: turn 1 finished in
//! full, then the same prompt again, stopped. Turn 1's reply must still be
//! its whole reply, not the second turn's text. (A real render of the same
//! thread, checked by eye, draws it whole too.)

use std::sync::{Arc, Mutex};

use ui::TestAppContext;

use super::headless::{mount, run_until};

/// The replies after the stop, once the run ends.
type Replies = Arc<Mutex<Option<Result<Vec<String>, String>>>>;

const LONG: &str = "Here is a full day in Lisbon: start in Alfama, lunch in Baixa, then Belém.";

#[ui::test]
fn a_stopped_second_turn_leaves_the_first_reply_whole(cx: &mut TestAppContext) {
    let (_runtime, engine, window, _shell, chat) = mount(cx);
    // Turn 1's whole reply; what follows is the test engine's own.
    engine.agents().reply(LONG);

    let replies: Replies = Arc::default();
    let into = Arc::clone(&replies);
    let chat_for = chat.clone();
    cx.spawn(move |mut cx| async move {
        let cx = &mut cx;
        let outcome = async {
            let first =
                super::new_chat_with(window, &chat_for, "Plan one day in Lisbon", cx).await?;
            let view = cx
                .update(|cx| chat_for.read(cx).thread(&first).cloned())
                .ok_or("no thread")?;
            cx.update(|cx| view.update(cx, |v, cx| v.send("Plan one day in Lisbon".into(), cx)));
            super::super::until_within(cx, &view, "turn 2's first text", super::UI_STEP, |v, _| {
                v.thread().turns().len() == 2 && !v.thread().turns()[1].reply().is_empty()
            })
            .await?;
            cx.update(|cx| view.update(cx, |v, cx| v.stop(cx)));
            super::super::until_within(cx, &view, "the stop", super::UI_STEP, |v, _| {
                v.thread()
                    .turns()
                    .last()
                    .is_some_and(|t| !t.state.is_open())
            })
            .await?;
            // Step 13's own check, on what the frame draws.
            let drawn = super::drawn::first_reply_whole(window, &view, cx)?;
            eprintln!("step 13: {drawn}");
            Ok(cx.update(|cx| {
                view.read(cx)
                    .thread()
                    .turns()
                    .iter()
                    .map(|t| t.reply())
                    .collect::<Vec<_>>()
            }))
        }
        .await;
        *into.lock().unwrap() = Some(outcome);
    })
    .detach();
    run_until(cx, std::slice::from_ref(&chat), |_| {
        replies.lock().unwrap().is_some()
    });
    let replies = replies.lock().unwrap().take().unwrap().expect("the run");
    eprintln!("replies after the stop: {replies:?}");
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0].trim(), LONG, "turn 1 keeps its whole reply");
}

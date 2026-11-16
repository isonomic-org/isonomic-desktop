//! The window checking itself, for a machine nobody can click on.
//!
//! `ISONOMIC_SELF_CHECK=<prompt>`: the real window, engine and model, in the
//! Capsule `ISONOMIC_CAPSULE_DIR` names. First run goes ahead once its model
//! is installed, Chat opens, and the prompt is typed into Chat's composer
//! and sent with Enter, as a person would. Then the same prompt again,
//! stopped at its first text and continued. Every step is timed and printed
//! to standard error; the window then stops the engine and exits 0 (PASS)
//! or 1 (FAIL).
//!
//! After the prompt check, Chat's sidebar toggle and next-chat keys go
//! through the window's own key dispatch, and whether each worked is
//! printed ([`keys`]).
//!
//! `ISONOMIC_SELF_CHECK=link`: the `isonomic://` link given as a launch
//! argument is opened; PASS when the app it names is on screen and, for
//! Chat, received it.
//!
//! It waits on notifications from the views it watches, never on a timer,
//! except for each step's time limit.

mod keys;

use std::time::{Duration, Instant};

use futures::{FutureExt as _, StreamExt as _};
use isonomic_client::Client;
use shell::apps::{link_segment, link_target, AppState};
use shell::first_run::{IdStep, Memory, ModelStep};
use shell::Shell;
use ui::{AnyWindowHandle, App, AsyncApp, Entity, Keystroke};

/// The environment variable that turns the check on.
pub const ENV: &str = "ISONOMIC_SELF_CHECK";
/// Set to `1` to let the check download first run's model when it isn't
/// installed, as on a machine that has only the disk image.
pub const DOWNLOAD: &str = "ISONOMIC_SELF_CHECK_DOWNLOAD";

fn download_allowed() -> bool {
    std::env::var(DOWNLOAD).is_ok_and(|v| v == "1")
}

/// How long one step may take: the engine's first start creates its
/// database, and a model on the Intel Mac loads in about ten seconds.
const STEP: Duration = Duration::from_secs(180);
/// How long the model download may take, when the check downloads it.
const DOWNLOAD_LIMIT: Duration = Duration::from_secs(1800);

const CHAT: &str = "com.isonomic.chat";

pub enum Mode {
    Prompt(String),
    Link,
}

pub fn mode() -> Option<Mode> {
    let value = std::env::var(ENV).ok()?;
    match value.trim() {
        "" => None,
        "link" => Some(Mode::Link),
        prompt => Some(Mode::Prompt(prompt.to_string())),
    }
}

/// Start the check once the window is open.
pub fn start(
    mode: Mode,
    window: AnyWindowHandle,
    shell: Entity<Shell>,
    client: Client,
    links: Vec<String>,
    cx: &mut App,
) {
    let started = Instant::now();
    say(started, "window up");
    cx.spawn(async move |cx| {
        let outcome = match mode {
            Mode::Prompt(prompt) => {
                prompt_check(&prompt, window, &shell, &client, started, cx).await
            }
            Mode::Link => link_check(links, window, &shell, &client, started, cx).await,
        };
        let code = match outcome {
            Ok(()) => {
                eprintln!("[self-check] PASS");
                0
            }
            Err(reason) => {
                eprintln!("[self-check] FAIL: {reason}");
                1
            }
        };
        // A scratch Capsule's engine stops with the check.
        let _ = client.shutdown().await;
        std::process::exit(code);
    })
    .detach();
}

pub(super) fn say(started: Instant, what: &str) {
    eprintln!(
        "[self-check] +{:.2}s {what}",
        started.elapsed().as_secs_f64()
    );
}

/// Wait until `done` holds for `entity`, waking on its notifications.
async fn until<T: 'static>(
    cx: &mut AsyncApp,
    entity: &Entity<T>,
    what: &str,
    done: impl FnMut(&T, &App) -> bool,
) -> Result<(), String> {
    until_within(cx, entity, what, STEP, done).await
}

/// [`until`] with its own time limit.
pub(super) async fn until_within<T: 'static>(
    cx: &mut AsyncApp,
    entity: &Entity<T>,
    what: &str,
    limit: Duration,
    mut done: impl FnMut(&T, &App) -> bool,
) -> Result<(), String> {
    let (wake, mut woken) = futures::channel::mpsc::unbounded::<()>();
    let _watch = cx.update(|cx| {
        cx.observe(entity, move |_, _| {
            let _ = wake.unbounded_send(());
        })
    });
    let deadline = Instant::now() + limit;
    loop {
        if cx.update(|cx| done(entity.read(cx), cx)) {
            return Ok(());
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!(
                "timed out after {}s waiting for {what}",
                STEP.as_secs()
            ));
        }
        let timer = cx.background_executor().timer(left).fuse();
        futures::pin_mut!(timer);
        futures::select! {
            _ = woken.next() => {}
            _ = timer => {}
        }
    }
}

/// Connected, and first run finished (or never shown).
async fn ready(
    window: AnyWindowHandle,
    shell: &Entity<Shell>,
    client: &Client,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    let mut live = client.live();
    let connected = live.wait_for(|live| live.connected).fuse();
    let timer = cx.background_executor().timer(STEP).fuse();
    futures::pin_mut!(connected, timer);
    futures::select! {
        seen = connected => { seen.map_err(|_| "the engine link stopped".to_string())?; }
        _ = timer => return Err("the engine never answered".into()),
    }
    say(started, "engine connected");

    let Some(first_run) = cx.update(|cx| shell.read(cx).first_run().cloned()) else {
        say(started, "first run already done");
        return Ok(());
    };
    until(
        cx,
        &first_run,
        "first run's memory check and model",
        |run, _| {
            !matches!(run.memory(), Memory::Checking)
                && !matches!(run.model(), ModelStep::Waiting | ModelStep::Choosing)
                || matches!(run.memory(), Memory::TooLittle(_) | Memory::Failed(_))
                || matches!(run.identity(), IdStep::Failed(_))
        },
    )
    .await?;
    let (memory, model) = cx.update(|cx| {
        let run = first_run.read(cx);
        (run.memory().clone(), run.model().clone())
    });
    let id = cx.update(|cx| first_run.read(cx).identity().clone());
    match &id {
        IdStep::Ready(value) => say(started, &format!("first run: Isonomic ID {value}")),
        IdStep::Failed(error) => return Err(format!("first run: no Isonomic ID: {error}")),
        _ => {}
    }
    match (&memory, &model) {
        (Memory::Enough(_), ModelStep::Installed(m)) => {
            say(
                started,
                &format!("first run: memory enough, {} installed", m.model),
            );
        }
        (Memory::Enough(_), ModelStep::Ready(m)) if download_allowed() => {
            let name = m.model.clone();
            say(started, &format!("first run: downloading {name}"));
            let downloading = Instant::now();
            let _ = window.update(cx, |_, _, cx| {
                first_run.update(cx, |run, cx| run.download_now(cx))
            });
            until_within(
                cx,
                &first_run,
                "the model download",
                DOWNLOAD_LIMIT,
                |run, _| {
                    matches!(
                        run.model(),
                        ModelStep::Installed(_) | ModelStep::Failed { .. }
                    )
                },
            )
            .await?;
            let step = cx.update(|cx| first_run.read(cx).model().clone());
            if let ModelStep::Failed { error, .. } = step {
                return Err(format!("first run: the download of {name} failed: {error}"));
            }
            say(
                started,
                &format!(
                    "first run: {name} downloaded in {:.0}s",
                    downloading.elapsed().as_secs_f64()
                ),
            );
        }
        (Memory::Enough(_), step) => {
            return Err(format!("first run: the model isn't installed ({step:?}); put it in the Capsule's models directory, or set {DOWNLOAD}=1"));
        }
        (memory, _) => return Err(format!("first run: {memory:?}")),
    }
    let _ = window.update(cx, |_, _, cx| {
        first_run.update(cx, |run, cx| run.continue_if_ready(cx))
    });
    until(cx, shell, "first run to finish", |shell, _| {
        shell.first_run().is_none()
    })
    .await?;
    say(started, "first run done");
    Ok(())
}

/// Open the app with `id` and wait until it is built.
async fn open(
    window: AnyWindowHandle,
    shell: &Entity<Shell>,
    id: &str,
    cx: &mut AsyncApp,
) -> Result<Entity<shell::apps::AppTab>, String> {
    let opened = window
        .update(cx, |_, window, cx| {
            shell.update(cx, |s, cx| s.open_app(id, window, cx))
        })
        .map_err(|e| e.to_string())?;
    if !opened {
        return Err(format!("{id} isn't mounted"));
    }
    let tab = cx
        .update(|cx| shell.read(cx).app(id).cloned())
        .ok_or_else(|| format!("{id} has no tab"))?;
    until(cx, &tab, "the app to open", |tab, _| {
        !matches!(tab.state(), AppState::Closed | AppState::Granting)
    })
    .await?;
    let failed = cx.update(|cx| match tab.read(cx).state() {
        AppState::Failed(error) => Some(error.clone()),
        _ => None,
    });
    if let Some(error) = failed {
        return Err(format!("{id} could not open: {error}"));
    }
    Ok(tab)
}

async fn prompt_check(
    prompt: &str,
    window: AnyWindowHandle,
    shell: &Entity<Shell>,
    client: &Client,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    ready(window, shell, client, started, cx).await?;
    let tab = open(window, shell, CHAT, cx).await?;
    let chat = cx
        .update(|cx| match tab.read(cx).state() {
            AppState::Open(app) => app.view().downcast::<isonomic_chat::Chat>().ok(),
            _ => None,
        })
        .ok_or("Chat's root view isn't Chat")?;
    say(started, "Chat mounted");
    let chat_shell = cx.update(|cx| chat.read(cx).shell().clone());

    // Chat's own path: its composer, then Enter.
    window
        .update(cx, |_, window, cx| {
            chat_shell.update(cx, |s, cx| s.set_draft(prompt, window, cx));
            window.dispatch_keystroke(Keystroke::parse("enter").expect("a key"), cx)
        })
        .map_err(|e| e.to_string())?;
    let sent = Instant::now();
    say(started, &format!("sent: {prompt:?}"));
    until(
        cx,
        &chat_shell,
        "Chat to open the new conversation",
        |s, _| matches!(s.route(), isonomic_chat::Route::Session(_)),
    )
    .await?;
    let view = cx
        .update(|cx| {
            let s = chat_shell.read(cx);
            match s.route() {
                isonomic_chat::Route::Session(id) => s.thread(id).cloned(),
                _ => None,
            }
        })
        .ok_or("Chat opened no conversation")?;

    // Turn one.
    until(cx, &view, "the first text", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| !t.reply().is_empty() || !t.state.is_open())
    })
    .await?;
    let first = sent.elapsed();
    until(cx, &view, "the end of the turn", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| !t.state.is_open())
    })
    .await?;
    let total = sent.elapsed();
    let (state, reply) = cx.update(|cx| {
        let v = view.read(cx);
        let thread = v.thread();
        let turn = thread.turns().last().expect("a turn");
        (format!("{:?}", turn.state), turn.reply())
    });
    eprintln!("[self-check] reply: {reply}");
    eprintln!(
        "[self-check] turn 1: first text {:.2}s, end {:.2}s, {state}",
        first.as_secs_f64(),
        total.as_secs_f64()
    );
    if !state.starts_with("Done") || reply.trim().is_empty() {
        return Err(format!(
            "turn 1 ended {state} with {} characters",
            reply.len()
        ));
    }

    // Turn two: stop at the first text, then continue.
    let _ = window.update(cx, |_, _, cx| {
        view.update(cx, |v, cx| v.send(prompt.to_string(), cx))
    });
    let sent = Instant::now();
    let turns = 2;
    until(cx, &view, "turn 2's first text", |v, _| {
        let thread = v.thread();
        thread.turns().len() >= turns
            && thread
                .turns()
                .last()
                .is_some_and(|t| !t.reply().is_empty() || !t.state.is_open())
    })
    .await?;
    let first = sent.elapsed();
    let _ = window.update(cx, |_, _, cx| view.update(cx, |v, cx| v.stop(cx)));
    until(cx, &view, "the stop", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| !t.state.is_open())
    })
    .await?;
    let (stopped_state, kept) = cx.update(|cx| {
        let v = view.read(cx);
        let thread = v.thread();
        let turn = thread.turns().last().expect("a turn");
        (format!("{:?}", turn.state), turn.reply().len())
    });
    eprintln!(
        "[self-check] turn 2: first text {:.2}s, stopped at {:.2}s with {kept} characters ({stopped_state})",
        first.as_secs_f64(),
        sent.elapsed().as_secs_f64()
    );
    let could = cx.update(|cx| {
        view.read(cx)
            .thread()
            .turns()
            .last()
            .is_some_and(|t| t.state.can_continue())
    });
    if !could {
        // The reply finished before the stop landed: nothing to continue.
        eprintln!("[self-check] turn 2 finished before the stop; continue not exercised");
        return Ok(());
    }
    // Continue is a request: the turn changes only when the engine answers,
    // so wait for it to move on from the stop before reading it.
    let stopped_at = cx.update(|cx| {
        view.read(cx)
            .thread()
            .turns()
            .last()
            .map(|t| t.revision)
            .unwrap_or(0)
    });
    let _ = window.update(cx, |_, _, cx| view.update(cx, |v, cx| v.resume(cx)));
    let resumed = Instant::now();
    until(cx, &view, "the continue to start", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| moved_on(stopped_at, t.revision))
    })
    .await?;
    until(cx, &view, "the continued text", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| continued(kept, t.reply().len(), t.state.is_open()))
    })
    .await?;
    let first = resumed.elapsed();
    until(cx, &view, "the end of the continued turn", |v, _| {
        v.thread()
            .turns()
            .last()
            .is_some_and(|t| !t.state.is_open())
    })
    .await?;
    let (state, reply, refusal) = cx.update(|cx| {
        let v = view.read(cx);
        let thread = v.thread();
        let turn = thread.turns().last().expect("a turn");
        (
            format!("{:?}", turn.state),
            turn.reply(),
            turn.refusal.clone(),
        )
    });
    if let Some(refusal) = refusal {
        return Err(format!("the engine refused to continue: {refusal}"));
    }
    eprintln!("[self-check] continued reply: {reply}");
    eprintln!(
        "[self-check] continue: first text {:.2}s, end {:.2}s, {state}",
        first.as_secs_f64(),
        resumed.elapsed().as_secs_f64()
    );
    if !state.starts_with("Done") || reply.len() <= kept {
        return Err(format!(
            "the continued turn ended {state} with {} characters (kept {kept})",
            reply.len()
        ));
    }
    keys::check(window, &chat_shell, started, cx)
        .await
        .map(|_| ())
}

async fn link_check(
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
    Ok(())
}

/// The turn changed after the person pressed Continue.
fn moved_on(stopped_at: u64, revision: u64) -> bool {
    revision > stopped_at
}

/// The continued reply has new text, or the turn ended. Only asked once the
/// turn has moved on from the stop ([`moved_on`]); before that, a stopped
/// turn would read as already ended.
fn continued(kept: usize, now: usize, open: bool) -> bool {
    now > kept || !open
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Intel Mac run: stopped at the first character, Continue pressed,
    /// and the check read the turn before the engine answered. The turn was
    /// still the stopped one (revision unchanged, not open, one character).
    #[test]
    fn a_stopped_turn_is_not_read_as_continued_until_it_moves_on() {
        let (stopped_at, kept) = (7, 1);
        // Before the engine answers: same revision, so the check waits.
        assert!(!moved_on(stopped_at, 7));
        // The old check asked only `continued`, which a stopped turn passes.
        assert!(continued(kept, 1, false));
        // Once the engine answers, the turn is open again with no new text
        // yet, and then has more.
        assert!(moved_on(stopped_at, 8));
        assert!(!continued(kept, 1, true));
        assert!(continued(kept, 40, true));
    }
}

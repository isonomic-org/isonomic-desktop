//! The full self-check's later steps: 9 the relaunch (in the link run),
//! 10 the model picker, 11 Acknowledgements, and the problem-report
//! notice's screenshot; and the state the prompt run hands the relaunch.

use std::sync::Arc;
use std::time::Instant;

use isonomic_chat::{ChatShell, Route};
use isonomic_client::Client;
use ui::component::WindowExt as _;
use ui::{AnyWindowHandle, AsyncApp, Entity};

use super::{shot, until_within, UI_STEP};

/// The variable through which the prompt run's state reaches the relaunch.
pub const ENV_EXPECT: &str = "ISONOMIC_SELF_CHECK_EXPECT";

/// What the relaunch must find: the same ID, as many chats, and the first
/// chat with as many turns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub id: String,
    pub chats: usize,
    pub chat: String,
    pub turns: usize,
}

impl State {
    /// The line the prompt run prints, and the variable's value.
    pub fn line(&self) -> String {
        format!(
            "id={} chats={} chat={} turns={}",
            self.id, self.chats, self.chat, self.turns
        )
    }

    pub fn parse(line: &str) -> Option<Self> {
        let field = |name: &str| {
            line.split_whitespace()
                .find_map(|part| part.strip_prefix(&format!("{name}=")))
                .map(str::to_string)
        };
        Some(Self {
            id: field("id")?,
            chats: field("chats")?.parse().ok()?,
            chat: field("chat")?,
            turns: field("turns")?.parse().ok()?,
        })
    }
}

/// The state now, at the end of the prompt run's full pass.
pub async fn state(
    chat: &Entity<ChatShell>,
    client: &Client,
    cx: &mut AsyncApp,
) -> Result<State, String> {
    let id = shell::first_run::identity_from(client).await?;
    cx.update(|cx| {
        let shell = chat.read(cx);
        let first = shell.sessions().first().ok_or("no chats")?;
        let turns = shell
            .thread(&first.id)
            .map(|v| v.read(cx).thread().turns().len())
            .unwrap_or(0);
        Ok(State {
            id,
            chats: shell.sessions().len(),
            chat: first.id.to_string(),
            turns,
        })
    })
}

/// Step 9: The relaunch on the same Capsule: no first run, the same ID, and the
/// chats and the first chat's history still there.
pub async fn relaunch(
    expected: &State,
    first_run_shown: bool,
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    client: &Client,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    if first_run_shown {
        return Err("first run was shown again".into());
    }
    let id = shell::first_run::identity_from(client).await?;
    if id != expected.id {
        return Err(format!("the ID is {id}, not {}", expected.id));
    }
    until_within(cx, chat, "the chats to load", UI_STEP, |s, _| s.is_loaded()).await?;
    let chats = cx.update(|cx| chat.read(cx).sessions().len());
    if chats != expected.chats {
        return Err(format!("{chats} chats, not {}", expected.chats));
    }
    let first: ui::SharedString = expected.chat.clone().into();
    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| s.open_session(first.clone(), window, cx))
        })
        .map_err(|e| e.to_string())?;
    let view = cx
        .update(|cx| chat.read(cx).thread(&first).cloned())
        .ok_or("the first chat has no thread")?;
    let turns = expected.turns;
    until_within(
        cx,
        &view,
        "the first chat's history",
        UI_STEP,
        move |v, _| v.thread().turns().len() == turns,
    )
    .await?;
    Ok(format!(
        "no first run; the same ID; {chats} chats; the first chat's {turns} turns"
    ))
}

/// Step 10: The model picker on Chat's home opens, with an installed model
/// picked. The list is opened by its trigger's id in the check build.
pub async fn picker(
    window: AnyWindowHandle,
    chat: &Entity<ChatShell>,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    window
        .update(cx, |_, window, cx| {
            chat.update(cx, |s, cx| s.new_chat(window, cx))
        })
        .map_err(|e| e.to_string())?;
    until_within(cx, chat, "Chat's home", UI_STEP, |s, _| {
        *s.route() == Route::Home
    })
    .await?;
    shot(window, "chat-home", true, started, cx);
    let (installed, picked) = cx.update(|cx| {
        let s = chat.read(cx);
        let installed: Vec<String> = s
            .catalog()
            .map(|c| {
                c.options()
                    .iter()
                    .filter(|o| o.availability.can_answer())
                    .map(|o| o.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        let picked = s.model().map(str::to_string).or_else(|| {
            s.catalog()
                .and_then(|c| c.recommended())
                .map(|o| o.id.clone())
        });
        (installed, picked)
    });
    let Some(picked) = picked.filter(|p| installed.contains(p)) else {
        return Err(format!(
            "the picked model is not installed (installed: {installed:?})"
        ));
    };
    open_picker(window, &picked, cx)?;
    shot(window, "model-picker", true, started, cx);
    let _ = super::press(window, "escape", cx);
    Ok(format!(
        "{picked} is installed and picked; the list opened with its row chosen"
    ))
}

/// Open the picker by its trigger, and say whether `picked`'s row is the
/// chosen one in the list that opened.
fn open_picker(window: AnyWindowHandle, picked: &str, cx: &mut AsyncApp) -> Result<(), String> {
    use ui::extra::model_picker::{list_id, row_id};
    let trigger: ui::ElementId = (ui::ElementId::from("model-picker"), "trigger").into();
    super::click(window, trigger, cx)?;
    chosen(
        window,
        list_id("model-picker"),
        row_id("model-picker", picked),
        cx,
    )
}

#[cfg(any(test, feature = "check-build"))]
fn chosen(
    window: AnyWindowHandle,
    list: ui::ElementId,
    row: ui::ElementId,
    cx: &mut AsyncApp,
) -> Result<(), String> {
    use ui::test::TestWindowExt as _;
    window
        .update(cx, |_, window, cx| {
            window.render_frame(cx);
            if !window.try_find(list).is_some_and(|l| l.visible()) {
                return Err("the model list did not open".to_string());
            }
            match window.try_find(row).map(|r| r.selected()) {
                Some(Some(true)) => Ok(()),
                Some(_) => Err("the picked model's row is not marked chosen".to_string()),
                None => Err("the picked model has no row in the list".to_string()),
            }
        })
        .map_err(|e| e.to_string())?
}

#[cfg(not(any(test, feature = "check-build")))]
fn chosen(
    _: AnyWindowHandle,
    _: ui::ElementId,
    _: ui::ElementId,
    _: &mut AsyncApp,
) -> Result<(), String> {
    Err(super::super::screens::NOT_THIS_BUILD.to_string())
}

/// The bundled programs, by the names their folders carry under
/// `Contents/Resources/sidecars`, and the notices the app ships.
fn bundled() -> Result<(Vec<String>, String), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let resources = exe
        .parent()
        .and_then(|p| p.parent())
        .ok_or("not inside an app")?
        .join("Resources");
    let notices = std::fs::read_to_string(resources.join("NOTICES.txt"))
        .map_err(|e| format!("Resources/NOTICES.txt: {e}"))?;
    let mut names: Vec<String> = std::fs::read_dir(resources.join("sidecars"))
        .map_err(|e| format!("Resources/sidecars: {e}"))?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .map(|n| {
            ["-x86_64-apple-darwin", "-aarch64-apple-darwin"]
                .iter()
                .find_map(|t| n.strip_suffix(t))
                .map_or(n.clone(), str::to_string)
        })
        .collect();
    names.sort();
    names.dedup();
    Ok((names, notices))
}

/// Step 11: Isonomic › Acknowledgements opens, and the notices name every
/// bundled program.
pub async fn acknowledgements(
    window: AnyWindowHandle,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    let (names, notices) = bundled()?;
    let missing: Vec<&String> = names
        .iter()
        .filter(|name| !notices.lines().any(|l| l.starts_with(&format!("{name} "))))
        .collect();
    window
        .update(cx, |_, window, cx| {
            window.dispatch_action(Box::new(crate::Acknowledgements), cx)
        })
        .map_err(|e| e.to_string())?;
    let open = async {
        for _ in 0..50 {
            let shown = window
                .update(cx, |_, window, cx| window.has_active_dialog(cx))
                .unwrap_or(false);
            if shown {
                return true;
            }
            cx.background_executor().timer(UI_STEP / 50).await;
        }
        false
    }
    .await;
    if !open {
        return Err("the Acknowledgements dialog did not open".into());
    }
    shot(window, "acknowledgements", true, started, cx);
    let _ = window.update(cx, |_, window, cx| window.close_dialog(cx));
    if !missing.is_empty() {
        return Err(format!("no notice for {missing:?}"));
    }
    Ok(format!("opened; notices for {}", names.join(", ")))
}

/// A problem-report notice, for its screenshot: a sample report kept in the
/// scratch Capsule, shown the way a real one is, then kept. Says whether
/// the notice opened.
pub async fn notice_screen(
    window: AnyWindowHandle,
    shell: &Entity<shell::Shell>,
    capsule: &std::path::Path,
    started: Instant,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    let sample = serde_json::json!({
        "format": 1, "kind": "app_error",
        "component": { "id": "com.isonomic.chat", "version": "self-check" },
        "message": "A sample report for the self-check's screenshot",
    });
    let store = reports::store::Store::in_capsule(capsule);
    store
        .save(&sample, 1)
        .map_err(|e| format!("keeping the sample report: {e}"))?;
    let kept = Arc::new(reports::kept::Kept::in_capsule(capsule));
    let _ = window.update(cx, |_, window, cx| {
        shell.update(cx, |s, cx| s.watch_reports(kept, cx));
        window.refresh();
    });
    for _ in 0..50 {
        let shown = window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap_or(false);
        if shown {
            shot(window, "problem-report-notice", true, started, cx);
            let _ = window.update(cx, |_, window, cx| window.close_dialog(cx));
            return Ok("the notice opened for a sample report".into());
        }
        cx.background_executor().timer(UI_STEP / 50).await;
    }
    Err(format!("no notice opened within {}s", UI_STEP.as_secs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_line_reads_back() {
        let state = State {
            id: "did:iso:abc".into(),
            chats: 2,
            chat: "s-1".into(),
            turns: 2,
        };
        assert_eq!(State::parse(&state.line()), Some(state.clone()));
        let printed = format!("[self-check] state {}", state.line());
        assert_eq!(State::parse(&printed), Some(state));
        assert_eq!(State::parse("id=x chats=two"), None);
    }
}

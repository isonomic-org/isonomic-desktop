// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Onboarding's calls on its backend, and what each answer changes: the
//! terms kept as accepted, the recovery kit and its file, the models'
//! list, and "Setting up your workspace", which keeps the choices and
//! fetches the models switched on.
//!
//! No call blocks the foreground thread, and each gives up after
//! [`WAIT`]: the engine may still be starting. What goes wrong is said
//! under the step's buttons, in the engine's own words where it has them.
//! The kit's words are never logged.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::future::{select, Either};
use futures::StreamExt as _;
use isonomic_proto::models::DownloadOutcome;
use ui::{ClipboardItem, Context};

use super::backend::{off_thread, Answer, Choices, Memory};
use super::engine::size;
use super::steps;
use super::Onboarding;

/// How long a call waits on the engine.
const WAIT: Duration = Duration::from_secs(30);
const NO_ANSWER: &str =
    "Isonomic\u{2019}s engine hasn\u{2019}t answered. It may still be starting: try again in a moment.";
/// What the kit's document is saved as, unless the person renames it.
pub const KIT_FILE: &str = "Isonomic Recovery Kit.json";
/// How long a failed download is said before onboarding goes on.
const SAID: Duration = Duration::from_secs(4);

impl Onboarding {
    /// Ask the backend, and give `then` its answer, or that none came. An
    /// answer only waits on the engine or on a thread of its own, so it
    /// is polled here: the background threads are busy with frames.
    pub(super) fn ask<T: Send + 'static>(
        &mut self,
        answer: Answer<T>,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, Result<T, String>, &mut Context<Self>) + 'static,
    ) {
        let wait = Box::pin(cx.background_executor().timer(WAIT));
        cx.spawn(async move |view, cx| {
            let result = match select(answer, wait).await {
                Either::Left((result, _)) => result,
                Either::Right(_) => Err(NO_ANSWER.to_owned()),
            };
            let _ = view.update(cx, |view, cx| {
                then(view, result, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// What the last call that failed said, if it is still shown.
    pub fn trouble(&self) -> Option<String> {
        self.trouble.as_ref().map(ToString::to_string)
    }

    /// The person's ID, once made.
    pub fn id(&self) -> Option<String> {
        self.account.as_ref().map(|account| account.id.clone())
    }

    /// Where the kit's file was saved, once it was.
    pub fn kit_file(&self) -> Option<&std::path::Path> {
        self.kit_file.as_deref()
    }

    /// What is said under "Setting up your workspace", if anything is.
    pub fn setting_up_says(&self) -> Option<String> {
        self.setup.as_ref().map(|(says, _)| says.to_string())
    }

    /// Say what went wrong, under the step's buttons, and log it.
    pub(super) fn troubled(&mut self, said: String) {
        reports::log::line(&format!("onboarding: {said}"));
        self.trouble = Some(said.into());
    }

    /// "Accept and continue" was pressed: the terms are kept as accepted.
    pub(super) fn keep_terms(&mut self, cx: &mut Context<Self>) {
        self.ask(self.backend.accept_terms(), cx, |_, kept, _| {
            if let Err(said) = kept {
                reports::log::line(&format!("onboarding: terms not kept: {said}"));
            }
        });
    }

    /// The models this Mac may run, for their step. Asked for as soon as
    /// the account exists, so the list is there when its step comes.
    pub fn load_models(&mut self, cx: &mut Context<Self>) {
        if self.catalog.is_some() || std::mem::replace(&mut self.catalog_asked, true) {
            return;
        }
        self.catalog_trouble = None;
        self.ask(self.backend.models(), cx, |view, catalog, _| {
            view.catalog_asked = false;
            match catalog {
                Ok(catalog) => {
                    let fits = catalog.memory == Memory::Enough;
                    view.models = catalog.models.iter().map(|m| m.on && fits).collect();
                    view.catalog = Some(catalog);
                }
                Err(said) => {
                    reports::log::line(&format!("onboarding: {said}"));
                    view.catalog_trouble = Some(said.into());
                }
            }
        });
    }

    /// The recovery kit, when its step comes on.
    pub fn fetch_kit(&mut self, cx: &mut Context<Self>) {
        if self.recovery.is_some() || std::mem::replace(&mut self.kit_asked, true) {
            return;
        }
        self.trouble = None;
        self.ask(self.backend.recovery_kit(), cx, |view, kit, _| {
            view.kit_asked = false;
            match kit {
                Ok(kit) => view.recovery = Some(kit),
                Err(said) => view.troubled(said),
            }
        });
    }

    /// Word `place` of the kit, counted from one; nothing until it is here.
    pub(super) fn word(&self, place: usize) -> String {
        self.recovery
            .as_ref()
            .and_then(|kit| kit.words.reveal().get(place.wrapping_sub(1)))
            .cloned()
            .unwrap_or_default()
    }

    /// "Save kit file": ask where, then write the kit's document there.
    pub fn save_kit(&mut self, cx: &mut Context<Self>) {
        if self.recovery.is_none() {
            return;
        }
        let documents = std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Documents"))
            .unwrap_or_default();
        let asked = cx.prompt_for_new_path(&documents, Some(KIT_FILE));
        cx.spawn(async move |view, cx| {
            if let Ok(Ok(Some(path))) = asked.await {
                let _ = view.update(cx, |view, cx| view.save_kit_to(path, cx));
            }
        })
        .detach();
    }

    /// Write the kit's document to `path`. It never holds the words.
    pub fn save_kit_to(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let Some(kit) = &self.recovery else {
            return;
        };
        let (document, to) = (kit.document.clone(), path.clone());
        let written = off_thread(move || std::fs::write(to, document).map_err(|e| e.to_string()));
        cx.spawn(async move |view, cx| {
            let written = written.await;
            let _ = view.update(cx, |view, cx| {
                match written {
                    Ok(()) => {
                        view.kit_file = Some(path);
                        view.trouble = None;
                    }
                    Err(error) => view.troubled(format!(
                        "The kit file couldn\u{2019}t be saved there ({error}). Choose another place."
                    )),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// "Copy": the ID goes to the pasteboard.
    pub fn copy_id(&mut self, cx: &mut Context<Self>) {
        if let Some(account) = &self.account {
            cx.write_to_clipboard(ClipboardItem::new_string(account.id.clone()));
            self.copied = true;
            cx.notify();
        }
    }

    /// What the person chose on the way.
    pub fn choices(&self) -> Choices {
        let models = self
            .catalog
            .iter()
            .flat_map(|catalog| &catalog.models)
            .zip(&self.models)
            .filter(|(_, on)| **on)
            .map(|(model, _)| model.id.clone())
            .collect();
        let integrations = steps::LINK_LIST
            .iter()
            .zip(self.links)
            .filter(|(_, on)| *on)
            .map(|((name, ..), _)| (*name).to_owned())
            .collect();
        Choices {
            models,
            integrations,
            start_at_login: self.residency[0],
            web_access: self.residency[1],
            snapshots: self.residency[2],
        }
    }

    /// "Setting up your workspace": keep the choices, then fetch the
    /// models switched on that this Mac hasn't got. The rest are started
    /// and left to the engine, whose downloads they are; the first is
    /// followed to its end, with how far it is said under the pill.
    pub(super) fn set_up(&mut self, cx: &mut Context<Self>) {
        let backend = Arc::clone(&self.backend);
        let choices = self.choices();
        let wanted: Vec<(String, String)> = self
            .catalog
            .iter()
            .flat_map(|catalog| &catalog.models)
            .zip(&self.models)
            .filter(|(model, on)| **on && !model.installed)
            .map(|(model, _)| (model.id.clone(), model.name.clone()))
            .collect();
        cx.spawn(async move |view, cx| {
            if let Err(said) = backend.finish(choices).await {
                reports::log::line(&format!("onboarding: choices not kept: {said}"));
            }
            // The rest first: each is started, and its first event is
            // enough to know the engine has it.
            let mut wanted = wanted.into_iter();
            let first = wanted.next();
            for (id, name) in wanted {
                if let Some(Err(said)) = backend.download(&id).next().await {
                    reports::log::line(&format!("onboarding: {name} wasn\u{2019}t downloaded: {said}"));
                }
            }
            let mut following = first.map(|(id, name)| (backend.download(&id), name));
            while let Some((events, name)) = &mut following {
                let Some(event) = events.next().await else {
                    break;
                };
                let (says, failed, ended) = match event {
                    Ok(event) => match (event.outcome, event.total_bytes) {
                        (None, total) => {
                            let of = total.map_or_else(String::new, |total| format!(" of {}", size(total)));
                            let so_far = size(event.received_bytes);
                            (format!("Downloading {name} \u{b7} {so_far}{of}"), false, false)
                        }
                        (Some(DownloadOutcome::Installed), _) => {
                            (format!("{name} is ready"), false, true)
                        }
                        (Some(_), _) => (
                            event.error.map_or_else(
                                || format!("{name} wasn\u{2019}t downloaded. Chat will offer it again."),
                                |error| format!("{name} wasn\u{2019}t downloaded: {}", error.message),
                            ),
                            true,
                            true,
                        ),
                    },
                    Err(said) => (format!("{name} wasn\u{2019}t downloaded: {said}"), true, true),
                };
                if failed {
                    reports::log::line(&format!("onboarding: {says}"));
                }
                let gone = view
                    .update(cx, |view, cx| {
                        view.setup = Some((says.into(), failed));
                        cx.notify();
                        view.handed_over
                    })
                    .unwrap_or(true);
                if gone {
                    return;
                }
                if failed {
                    cx.background_executor().timer(SAID).await;
                }
                if ended {
                    break;
                }
            }
            let _ = view.update(cx, |view, cx| {
                view.set_up_done = true;
                view.arrive(cx);
            });
        })
        .detach();
    }

    /// "Continue while it downloads": the download is the engine's, and
    /// goes on without this window.
    pub fn leave_it_downloading(&mut self, cx: &mut Context<Self>) {
        self.set_up_done = true;
        self.arrive(cx);
    }

    /// Hand over, once the workspace is set up and its screen has played.
    pub(super) fn arrive(&mut self, cx: &mut Context<Self>) {
        if self.set_up_done && self.setup_shown {
            self.hand_over(cx);
        }
        cx.notify();
    }
}

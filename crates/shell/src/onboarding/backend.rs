// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! What onboarding asks of the engine and of this Mac: the terms kept as
//! accepted, the Isonomic ID, its recovery kit, the local models and their
//! downloads, and the choices made on the last step.
//!
//! The window's is [`super::engine::EngineBackend`]. [`Sample`] answers
//! with the design's own samples, for tests and for drawing the screens
//! with no engine.
//!
//! Every refusal is plain words, shown to the person as written.

use std::sync::Mutex;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::StreamExt as _;
use isonomic_proto::identity::RecoveryWords;
use isonomic_proto::models::{DownloadOutcome, DownloadProgress};

use super::steps;

/// The person's Isonomic ID, and whether this call made it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub created: bool,
}

/// The recovery kit: the words the person writes down, and the document
/// the window saves as a file. The document never holds the words.
#[derive(Debug, Clone)]
pub struct Kit {
    pub id: String,
    pub words: RecoveryWords,
    pub document: String,
}

/// A local model, as its row shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub about: String,
    /// The tab it is under, by its place in [`steps::MODEL_TABS`].
    pub kind: usize,
    /// Does its switch start on?
    pub on: bool,
    pub installed: bool,
}

/// Has this Mac the memory local models need?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Memory {
    Enough,
    Short { has: u64, needs: u64 },
}

/// The local models this Mac may run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub models: Vec<Model>,
    pub memory: Memory,
}

/// What the person chose on the way: the models switched on, the
/// integrations asked for, and the last step's three switches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Choices {
    pub models: Vec<String>,
    pub integrations: Vec<String>,
    pub start_at_login: bool,
    pub web_access: bool,
    pub snapshots: bool,
}

pub type Answer<T> = BoxFuture<'static, Result<T, String>>;

/// `work` on a thread of its own, for what blocks on a file or the system:
/// an answer is polled on the foreground thread, which must never wait,
/// and the shared background threads are busy drawing onboarding's frames.
/// Not for a backend a test runs: a test's scheduler takes no waking from
/// a thread it doesn't know.
pub fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Answer<T> {
    let (done, wait) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = done.send(work());
    });
    Box::pin(async move {
        wait.await
            .unwrap_or_else(|_| Err("Isonomic stopped before this was done.".to_owned()))
    })
}

pub trait Backend: Send + Sync + 'static {
    /// Keep that the terms were accepted, and when.
    fn accept_terms(&self) -> Answer<()>;
    /// The person's ID, made now if this Mac has none.
    fn create_account(&self) -> Answer<Account>;
    /// The recovery kit, which the person asked for by reaching its step.
    fn recovery_kit(&self) -> Answer<Kit>;
    /// The person typed the kit's words back: it is saved.
    fn kit_saved(&self) -> Answer<()>;
    fn models(&self) -> Answer<Catalog>;
    /// Download `model`, or follow its running download. The stream ends
    /// after the event that has an outcome; dropping it only stops
    /// following.
    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>>;
    /// Does Isonomic start at login now? `None` where this build can't.
    fn starts_at_login(&self) -> Option<bool>;
    /// Keep the choices and apply the ones this build can.
    fn finish(&self, choices: Choices) -> Answer<()>;
}

/// The ID the design's samples stand in for.
pub const SAMPLE_ID: &str = "isonomic:7Q4K-M2XD-9PLV";

/// The design's samples, with nothing behind them. A call named to
/// [`Sample::refuse`] fails until [`Sample::recover`]; every call is
/// noted, for a test to read.
#[derive(Default)]
pub struct Sample {
    refused: Mutex<Vec<(&'static str, String)>>,
    calls: Mutex<Vec<String>>,
    short: Mutex<Option<Memory>>,
    finished: Mutex<Option<Choices>>,
}

impl Sample {
    pub fn refuse(&self, call: &'static str, says: &str) {
        self.refused.lock().unwrap().push((call, says.to_owned()));
    }

    pub fn recover(&self) {
        self.refused.lock().unwrap().clear();
    }

    /// Answer `models` as a Mac with this memory would.
    pub fn with_memory(&self, memory: Memory) {
        *self.short.lock().unwrap() = Some(memory);
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    /// What `finish` was last given.
    pub fn choices(&self) -> Option<Choices> {
        self.finished.lock().unwrap().clone()
    }

    fn answer<T: Send + 'static>(&self, call: &'static str, with: T) -> Answer<T> {
        self.calls.lock().unwrap().push(call.to_owned());
        let refused = self
            .refused
            .lock()
            .unwrap()
            .iter()
            .find(|(name, _)| *name == call)
            .map(|(_, says)| says.clone());
        Box::pin(async move { refused.map_or(Ok(with), Err) })
    }
}

/// What `choices` is kept as.
pub fn kept(choices: &Choices) -> serde_json::Value {
    serde_json::json!({
        "models": choices.models,
        "integrations": choices.integrations,
        "start_at_login": choices.start_at_login,
        "web_access": choices.web_access,
        "snapshots": choices.snapshots,
    })
}

impl Backend for Sample {
    fn accept_terms(&self) -> Answer<()> {
        self.answer("accept_terms", ())
    }

    fn create_account(&self) -> Answer<Account> {
        let account = Account {
            id: SAMPLE_ID.to_owned(),
            created: true,
        };
        self.answer("create_account", account)
    }

    fn recovery_kit(&self) -> Answer<Kit> {
        let kit = Kit {
            id: SAMPLE_ID.to_owned(),
            words: RecoveryWords::new(steps::WORDS.map(str::to_owned).to_vec()),
            document: format!("{{\"id\":\"{SAMPLE_ID}\",\"sample\":true}}\n"),
        };
        self.answer("recovery_kit", kit)
    }

    fn kit_saved(&self) -> Answer<()> {
        self.answer("kit_saved", ())
    }

    fn models(&self) -> Answer<Catalog> {
        let models = steps::MODEL_LIST
            .iter()
            .zip(steps::MODEL_KINDS)
            .map(|((name, about, on), kind)| Model {
                id: name.to_lowercase().replace(' ', "-"),
                name: (*name).to_owned(),
                about: (*about).to_owned(),
                kind,
                on: *on,
                installed: false,
            })
            .collect();
        let memory = self.short.lock().unwrap().unwrap_or(Memory::Enough);
        self.answer("models", Catalog { models, memory })
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        self.calls.lock().unwrap().push(format!("download {model}"));
        let refused = self
            .refused
            .lock()
            .unwrap()
            .iter()
            .find(|(name, _)| *name == "download")
            .map(|(_, says)| says.clone());
        let done = DownloadProgress {
            model: model.to_owned(),
            received_bytes: 1,
            total_bytes: Some(1),
            outcome: Some(DownloadOutcome::Installed),
            error: None,
        };
        futures::stream::once(async move { refused.map_or(Ok(done), Err) }).boxed()
    }

    fn starts_at_login(&self) -> Option<bool> {
        None
    }

    fn finish(&self, choices: Choices) -> Answer<()> {
        *self.finished.lock().unwrap() = Some(choices);
        self.answer("finish", ())
    }
}

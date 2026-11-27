// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! First run's setup over the engine client and the Capsule's marker. A
//! self-check still goes through first run; a person meets onboarding.

use engine_link::setup::Capsule;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_client::Client;
use isonomic_proto::models::{DownloadProgress, Recommendation};
use isonomic_proto::HardwareProfile;
use shell::downloads;
use shell::first_run::{self, Setup};

/// First run through the engine client and the Capsule's marker.
pub struct FirstRunSetup {
    pub client: Client,
    pub capsule: Capsule,
}

impl Setup for FirstRunSetup {
    fn profile(&self) -> BoxFuture<'static, Result<HardwareProfile, String>> {
        first_run::profile_from(&self.client)
    }

    fn recommended(&self) -> BoxFuture<'static, Result<Recommendation, String>> {
        downloads::recommended(&self.client)
    }

    fn installed(&self, model: &str) -> BoxFuture<'static, Result<bool, String>> {
        downloads::installed(&self.client, model)
    }

    fn model_name(&self, model: &str) -> BoxFuture<'static, Option<String>> {
        let (client, model) = (self.client.clone(), model.to_string());
        Box::pin(async move {
            let models = client.models().list().await.ok()?;
            // Named as Chat names it: without the quantisation tag.
            models
                .into_iter()
                .find(|m| m.id == model)
                .map(|m| conversation_model::display_name(&m.name))
        })
    }

    fn download(&self, model: &str) -> BoxStream<'static, Result<DownloadProgress, String>> {
        downloads::download(&self.client, model)
    }

    fn cancel_download(&self, model: &str) -> BoxFuture<'static, Result<(), String>> {
        downloads::cancel(&self.client, model)
    }

    fn identity(&self) -> BoxFuture<'static, Result<String, String>> {
        first_run::identity_from(&self.client)
    }

    fn finish(&self) -> BoxFuture<'static, Result<(), String>> {
        let capsule = self.capsule.clone();
        Box::pin(async move { capsule.mark_set_up().map_err(|e| e.to_string()) })
    }
}

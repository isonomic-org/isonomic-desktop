//! First run's model calls over the shell's client: the recommended local
//! model, whether it is installed, and its download with progress.
//!
//! The engine owns a download. Dropping the stream only stops following it;
//! downloading the same model again attaches to the running one.

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use isonomic_client::Client;
use isonomic_proto::models::{DownloadProgress, ModelState, Recommendation};

/// The local model to start with on this machine.
pub fn recommended(client: &Client) -> BoxFuture<'static, Result<Recommendation, String>> {
    let models = client.models();
    Box::pin(async move { models.recommended(None).await.map_err(|e| e.to_string()) })
}

/// Whether `model`'s file is in place.
pub fn installed(client: &Client, model: &str) -> BoxFuture<'static, Result<bool, String>> {
    let models = client.models();
    let model = model.to_string();
    Box::pin(async move {
        let detail = models.info(&model).await.map_err(|e| e.to_string())?;
        Ok(!matches!(
            detail.state,
            ModelState::NotInstalled | ModelState::Downloading
        ))
    })
}

/// Download `model`, or attach to its running download. The stream ends
/// after the event that has an outcome.
pub fn download(
    client: &Client,
    model: &str,
) -> BoxStream<'static, Result<DownloadProgress, String>> {
    let models = client.models();
    let model = model.to_string();
    let started = async move { models.download(&model).await.map_err(|e| e.to_string()) };
    let stream = futures::stream::once(started).flat_map(|started| match started {
        Ok(stream) => futures::stream::unfold(stream, |mut stream| async move {
            let next = stream.next().await?;
            Some((next.map_err(|e| e.to_string()), stream))
        })
        .boxed(),
        Err(error) => futures::stream::once(async move { Err(error) }).boxed(),
    });
    Box::pin(stream)
}

/// Stop `model`'s download for every window following it.
pub fn cancel(client: &Client, model: &str) -> BoxFuture<'static, Result<(), String>> {
    let models = client.models();
    let model = model.to_string();
    Box::pin(async move {
        models
            .cancel_download(&model)
            .await
            .map_err(|e| e.to_string())
    })
}

use futures::StreamExt as _;

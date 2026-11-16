//! Sending a kept report (problem-report design, part 3): one HTTPS POST
//! of the report's bytes, exactly as Show report displays them, to one
//! configured URL. The receiver's side is `docs/architecture/window/report-receiver.md`.
//!
//! - **Where:** [`Destination::configured`]. A release sets
//!   `ISONOMIC_REPORTS_URL`, and optionally `ISONOMIC_REPORTS_KEY`, when it
//!   compiles the window; a test build may set the same names when it
//!   runs. With no URL nothing is offered and nothing is sent.
//! - **What:** the body, with `Content-Type: application/json`, the
//!   report's id (its own `id`, or [`derived_id`] for an older one) in
//!   `Idempotency-Key`, and the key, if any, as `Authorization: Bearer`.
//! - **Retries** ([`next_wait`]): network errors and 5xx are tried again
//!   with a doubling wait, for a day; a 4xx never is.
//!
//! Nothing here logs a report's body: only status codes and error kinds.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{header, Request};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use sha2::{Digest, Sha256};

/// The environment variables a release compiles in, and a test build may
/// set at run time instead.
pub const ENV_URL: &str = "ISONOMIC_REPORTS_URL";
pub const ENV_KEY: &str = "ISONOMIC_REPORTS_KEY";
/// The header that carries a report's id.
pub const ID_HEADER: &str = "Idempotency-Key";
/// The largest body sent; the receiver may refuse more with 413.
pub const MAX_BYTES: usize = 64 * 1024;
/// How long one attempt may take.
pub const ATTEMPT_LIMIT: Duration = Duration::from_secs(30);
/// The first wait before trying again; each later one doubles.
pub const FIRST_WAIT: Duration = Duration::from_secs(30);
/// The longest wait between two attempts.
pub const LONGEST_WAIT: Duration = Duration::from_secs(3600);
/// A report that hasn't gone after this long is kept, and not tried again
/// until the person sends it again.
pub const GIVE_UP_AFTER: Duration = Duration::from_secs(24 * 3600);

/// Where reports go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub url: String,
    /// Sent as `Authorization: Bearer <key>`.
    pub key: Option<String>,
}

impl Destination {
    /// The destination this build is configured with: the run-time
    /// variables when set (a test build), else what the release compiled
    /// in. `None` when there is no URL, or it is not HTTPS (plain HTTP is
    /// allowed to this Mac only, for a local test server).
    pub fn configured() -> Option<Self> {
        let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        let compiled = |value: Option<&str>| value.filter(|v| !v.is_empty()).map(String::from);
        let (url, key) = match env(ENV_URL) {
            Some(url) => (url, env(ENV_KEY)),
            None => (
                compiled(option_env!("ISONOMIC_REPORTS_URL"))?,
                compiled(option_env!("ISONOMIC_REPORTS_KEY")),
            ),
        };
        Self::new(url, key)
    }

    /// `url` if it may be sent to.
    pub fn new(url: String, key: Option<String>) -> Option<Self> {
        allowed(&url).then_some(Self { url, key })
    }

    /// The host, for the setting's line ("Reports go to …").
    pub fn host(&self) -> &str {
        let rest = self.url.split_once("://").map_or(&*self.url, |(_, r)| r);
        rest.split(['/', '?', '#']).next().unwrap_or(rest)
    }
}

fn allowed(url: &str) -> bool {
    if url.starts_with("https://") {
        return url.len() > "https://".len();
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = host.rsplit_once(':').map_or(host, |(h, _)| h);
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

/// The id of a report kept before format 1 carried one: the first 128
/// bits of the SHA-256, in hex, of the Capsule's id and the file's name
/// (without `.json`). The same report keeps it across retries and
/// launches, so the receiver can drop a repeat; two reports from one Mac
/// share nothing a receiver could link. It has the form of the SDK's ids
/// (`isonomic_reports::id::is_id`); a newer report has its own, random.
pub fn derived_id(capsule_id: &str, file_name: &str) -> String {
    let stem = file_name.strip_suffix(".json").unwrap_or(file_name);
    Sha256::digest(format!("{capsule_id}/{stem}").as_bytes())
        .iter()
        .take(16)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// How one attempt ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attempt {
    /// 2xx: the receiver has it. The kept file can go.
    Sent,
    /// 4xx: never tried again. The report stays kept.
    Refused(u16),
    /// A network error, a timeout or a 5xx: worth trying again.
    Failed(String),
}

/// How long to wait before attempt `tries + 1`, `since_first` after the
/// first; `None` once a day has passed.
pub fn next_wait(tries: u32, since_first: Duration) -> Option<Duration> {
    let doubled = FIRST_WAIT.saturating_mul(1u32 << tries.saturating_sub(1).min(16));
    let wait = doubled.min(LONGEST_WAIT);
    (since_first + wait <= GIVE_UP_AFTER).then_some(wait)
}

/// Sends a body somewhere. The window's is [`post`] on its runtime; tests
/// script one.
pub trait Sender: Send + Sync + 'static {
    fn send(&self, body: String, id: String) -> Pin<Box<dyn Future<Output = Attempt> + Send>>;
}

/// One attempt: POST `body`, the report with `id`, to `to`. Needs a Tokio
/// runtime.
pub async fn post(to: &Destination, body: String, id: String) -> Attempt {
    if body.len() > MAX_BYTES {
        return Attempt::Refused(413);
    }
    let connector = match hyper_rustls::HttpsConnectorBuilder::new().with_native_roots() {
        Ok(builder) => builder.https_or_http().enable_http1().build(),
        Err(error) => return Attempt::Failed(format!("no root certificates: {}", error.kind())),
    };
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);
    let mut request = Request::post(&to.url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(ID_HEADER, id);
    if let Some(key) = &to.key {
        request = request.header(header::AUTHORIZATION, format!("Bearer {key}"));
    }
    let request = match request.body(Full::new(Bytes::from(body))) {
        Ok(request) => request,
        Err(_) => return Attempt::Refused(400),
    };
    match tokio::time::timeout(ATTEMPT_LIMIT, client.request(request)).await {
        Err(_) => Attempt::Failed("timed out".into()),
        Ok(Err(error)) => Attempt::Failed(if error.is_connect() {
            "could not connect".into()
        } else {
            "the connection failed".into()
        }),
        Ok(Ok(response)) => match response.status().as_u16() {
            200..=299 => Attempt::Sent,
            code @ 400..=499 => Attempt::Refused(code),
            code => Attempt::Failed(format!("status {code}")),
        },
    }
}

/// The window's sender: [`post`] on a one-thread runtime of its own, so
/// sending shares nothing with the engine link.
pub struct HttpSender {
    to: Destination,
    runtime: tokio::runtime::Runtime,
}

impl HttpSender {
    pub fn new(to: Destination) -> std::io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("isonomic-report-sender")
            .enable_all()
            .build()?;
        Ok(Self { to, runtime })
    }

    pub fn destination(&self) -> &Destination {
        &self.to
    }
}

impl Sender for HttpSender {
    fn send(&self, body: String, id: String) -> Pin<Box<dyn Future<Output = Attempt> + Send>> {
        let to = self.to.clone();
        let task = self.runtime.spawn(async move { post(&to, body, id).await });
        Box::pin(async move {
            task.await
                .unwrap_or_else(|_| Attempt::Failed("the sender stopped".into()))
        })
    }
}

//! The window's link to the engine (architecture spec, sections 3 and 6.1).
//!
//! One connection per process. This crate works out where the engine's
//! socket is, starts the engine when nothing is listening there, and hands
//! out a client that reconnects on its own. It knows nothing about windows
//! or views: the shell receives a `Client` and a watch on the live status.
//!
//! The connection runs on a small Tokio runtime owned by the link. The
//! client's methods and its live watch can be awaited from the interface
//! framework's own executor.
//!
//! From an app bundle on macOS, the engine is started through the login
//! agent (`agent`), so the engine the window starts is one the agent
//! restarts after a crash.

pub mod agent;
pub mod problems;
pub mod settings;
pub mod setup;

use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use agent::{in_app_bundle, LoginAgent, SystemLaunchctl};
use isonomic_client::{unix_connector, Client, Connector, Options};
use isonomic_proto::locate;
use problems::Problems;

/// Path of the engine binary, when it is not next to the window binary.
pub const ENV_ENGINE_BIN: &str = "ISONOMIC_ENGINE_BIN";
/// File name of the engine binary beside the window binary.
pub const ENGINE_BIN_NAME: &str = "isonomic-capsule";
/// Do not try to start the engine more often than this.
pub const RELAUNCH_AFTER: Duration = Duration::from_secs(10);

/// Starts the engine. The real one spawns the binary; tests script it.
pub trait Launcher: Send + Sync + 'static {
    fn launch(&self) -> io::Result<()>;
}

impl<F: Fn() -> io::Result<()> + Send + Sync + 'static> Launcher for F {
    fn launch(&self) -> io::Result<()> {
        self()
    }
}

/// Launches the engine binary as its own process group, so it keeps running
/// when the window exits. Its environment is passed through unchanged: the
/// engine reads only its own `ISONOMIC_` variables and builds every child's
/// environment from an allow-list.
pub struct EngineBinary {
    pub path: PathBuf,
    /// Where its standard error goes (appended), so an engine that dies on
    /// start leaves its reason. `None` discards it.
    pub log: Option<PathBuf>,
    /// Told when the engine can't start or exits with a failure.
    pub problems: Problems,
}

impl Launcher for EngineBinary {
    fn launch(&self) -> io::Result<()> {
        let mut command = std::process::Command::new(&self.path);
        let stderr = match &self.log {
            Some(log) => {
                if let Some(dir) = log.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log)?;
                std::process::Stdio::from(file)
            }
            None => std::process::Stdio::null(),
        };
        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(stderr);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command.spawn().inspect_err(|error| {
            self.problems
                .report(problems::not_started(&self.path, error));
        })?;
        // Reap it if it exits while the window is still alive, and say why
        // when it failed.
        let (problems, log) = (self.problems.clone(), self.log.clone());
        std::thread::spawn(move || {
            if let Ok(status) = child.wait() {
                if !status.success() {
                    problems.report(problems::exited(&status, log.as_deref()));
                }
            }
        });
        Ok(())
    }
}

/// Starts the engine through the login agent when it's installed, and
/// spawns the binary otherwise (agent off, or not loaded).
pub struct AgentFirst {
    pub agent: Arc<LoginAgent>,
    pub binary: EngineBinary,
}

impl Launcher for AgentFirst {
    fn launch(&self) -> io::Result<()> {
        match self.agent.kickstart() {
            Ok(true) => Ok(()),
            _ => self.binary.launch(),
        }
    }
}

/// Wrap a connector so that a failed attempt asks the launcher to start the
/// engine, at most once per `relaunch_after`. The attempt itself still
/// fails; the client's own reconnect loop picks the engine up once it
/// listens.
pub fn launching_connector(
    inner: Connector,
    launcher: Arc<dyn Launcher>,
    relaunch_after: Duration,
    problems: Problems,
) -> Connector {
    let last_launch: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    Arc::new(move || {
        let attempt = inner();
        let launcher = Arc::clone(&launcher);
        let last_launch = Arc::clone(&last_launch);
        let problems = problems.clone();
        Box::pin(async move {
            let result = attempt.await;
            if result.is_ok() {
                problems.clear();
            } else {
                let mut last = last_launch.lock().expect("launch lock");
                let due = last
                    .map(|at| at.elapsed() >= relaunch_after)
                    .unwrap_or(true);
                if due {
                    *last = Some(Instant::now());
                    let _ = launcher.launch();
                }
            }
            result
        })
    })
}

#[derive(Debug, thiserror::Error)]
pub enum LinkError {
    #[error(
        "cannot locate the Capsule directory: set {} or HOME",
        locate::ENV_CAPSULE_DIR
    )]
    NoCapsuleDir,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

/// Where the socket is and which binary to start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkConfig {
    /// The Capsule directory.
    pub root: PathBuf,
    pub socket: PathBuf,
    /// `None` means never start the engine, only connect to it.
    pub engine: Option<PathBuf>,
    /// `~/Library/LaunchAgents`, when there is a home directory.
    pub launch_agents: Option<PathBuf>,
    pub user_id: u32,
    pub client_name: String,
}

#[cfg(unix)]
fn user_id() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

#[cfg(not(unix))]
fn user_id() -> u32 {
    0
}

impl LinkConfig {
    /// Work the configuration out from plain inputs. Pure, so tests pass
    /// their own.
    pub fn resolve(
        capsule_override: Option<std::ffi::OsString>,
        home: Option<PathBuf>,
        engine_override: Option<PathBuf>,
        window_binary: Option<PathBuf>,
        user_id: u32,
    ) -> Result<Self, LinkError> {
        let root = locate::capsule_root(capsule_override.as_deref(), home.as_deref())
            .ok_or(LinkError::NoCapsuleDir)?;
        let beside_window =
            window_binary.and_then(|exe| exe.parent().map(|dir| dir.join(ENGINE_BIN_NAME)));
        Ok(Self {
            socket: locate::engine_socket(&root, user_id),
            root,
            engine: engine_override.or(beside_window),
            launch_agents: home.map(|home| home.join("Library/LaunchAgents")),
            user_id,
            client_name: "isonomic-desktop".to_string(),
        })
    }

    /// The Capsule directory, for first-run setup.
    pub fn capsule(&self) -> setup::Capsule {
        setup::Capsule::new(self.root.clone())
    }

    /// The login agent, on macOS when the engine is inside an app bundle.
    pub fn login_agent(&self) -> Option<LoginAgent> {
        if !cfg!(target_os = "macos") {
            return None;
        }
        let engine = self.engine.clone().filter(|path| in_app_bundle(path))?;
        Some(LoginAgent::new(
            self.launch_agents.clone()?,
            self.root.clone(),
            engine,
            self.user_id,
            Arc::new(SystemLaunchctl),
        ))
    }

    /// The configuration for this process.
    pub fn from_environment() -> Result<Self, LinkError> {
        Self::resolve(
            std::env::var_os(locate::ENV_CAPSULE_DIR),
            std::env::var_os("HOME").map(PathBuf::from),
            std::env::var_os(ENV_ENGINE_BIN).map(PathBuf::from),
            std::env::current_exe().ok(),
            user_id(),
        )
    }
}

/// Whether the window runs from a path macOS made up for an app that was
/// opened without being moved first (App Translocation): a quarantined,
/// unnotarized download opened from the disk image or Downloads. Its path
/// changes every launch, so nothing may point at it.
pub fn translocated(exe: &std::path::Path) -> bool {
    exe.components()
        .any(|part| part.as_os_str() == "AppTranslocation")
}

/// The running link. Keep it alive for as long as the window process lives.
pub struct EngineLink {
    client: Client,
    problems: Problems,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Drop for EngineLink {
    fn drop(&mut self) {
        // Never block on shutdown: the link may be dropped from async code.
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

impl EngineLink {
    /// Start connecting in the background and return at once. With a login
    /// agent, a missing engine is started through it.
    pub fn start(config: LinkConfig, agent: Option<Arc<LoginAgent>>) -> Result<Self, LinkError> {
        let problems = Problems::default();
        let mut connector = unix_connector(config.socket);
        if let Some(path) = config.engine {
            let binary = EngineBinary {
                path,
                // The engine's own log is `logs/engine.log`, which it writes
                // once it has accepted the Capsule; before that it only has
                // standard error.
                log: Some(config.root.join("logs").join("engine.stderr")),
                problems: problems.clone(),
            };
            let launcher: Arc<dyn Launcher> = match agent {
                Some(agent) => Arc::new(AgentFirst { agent, binary }),
                None => Arc::new(binary),
            };
            connector = launching_connector(connector, launcher, RELAUNCH_AFTER, problems.clone());
        }
        let mut link = Self::with_connector(connector, Options::new(config.client_name))?;
        link.problems = problems;
        Ok(link)
    }

    /// Start with any connector. Tests pass the in-memory engine's.
    pub fn with_connector(connector: Connector, options: Options) -> Result<Self, LinkError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("engine-link")
            .enable_all()
            .build()?;
        let client = Client::start(runtime.handle(), connector, options);
        Ok(Self {
            client,
            problems: Problems::default(),
            runtime: Some(runtime),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Why the engine can't be reached, in plain words; `None` once
    /// connected.
    pub fn problems(&self) -> tokio::sync::watch::Receiver<Option<String>> {
        self.problems.watch()
    }
}

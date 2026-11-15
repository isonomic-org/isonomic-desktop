//! The Milestone 0 gate, driven through the SDK client exactly as the window
//! drives the engine: start, wait for healthy, force-kill PostgreSQL and
//! watch it come back, then ask for an ordered shutdown and check that
//! nothing is left running.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use isonomic_client::{unix_connector, Client, Live, Options};
use isonomic_proto::system::{ServiceState, ServiceStatus};
use tokio::sync::watch;

/// First start includes first-time database setup and, on macOS, the
/// system's one-off check of programs it has not run before.
const START_TIMEOUT: Duration = Duration::from_secs(180);
const RECOVERY_TIMEOUT: Duration = Duration::from_secs(90);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60);
/// How long processes may take to disappear after their parent has gone.
const REAP_GRACE: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(50);
const POSTGRES: &str = "postgres";

/// How to run the engine under test.
pub struct Engine {
    pub program: PathBuf,
    /// `ISONOMIC_BUNDLE_DIR`, when the engine is not inside an app.
    pub bundle_dir: Option<PathBuf>,
}

/// One gate item.
#[derive(Debug, Clone)]
pub struct Check {
    pub passed: bool,
    pub detail: String,
}

impl Check {
    fn pass(detail: impl Into<String>) -> Self {
        Self {
            passed: true,
            detail: detail.into(),
        }
    }
    fn fail(detail: impl Into<String>) -> Self {
        Self {
            passed: false,
            detail: detail.into(),
        }
    }
}

/// What the run saw.
#[derive(Debug, Default)]
pub struct Outcome {
    pub started: Option<Check>,
    pub recovered: Option<Check>,
    pub stopped: Option<Check>,
    pub engine_version: Option<String>,
    pub cold_start_ms: Option<u64>,
    pub healthy_after: Option<Duration>,
    pub profile: Option<String>,
    pub memory_bytes: Option<u64>,
    pub engine_log: PathBuf,
    pub postgres_log: PathBuf,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        [&self.started, &self.recovered, &self.stopped]
            .iter()
            .all(|c| c.as_ref().is_some_and(|c| c.passed))
    }
}

pub async fn run(engine: &Engine, scratch: &Path) -> Outcome {
    let capsule = scratch.join("capsule");
    let mut outcome = Outcome {
        engine_log: scratch.join("engine.log"),
        postgres_log: capsule.join("logs").join(format!("{POSTGRES}.log")),
        ..Outcome::default()
    };
    let mut child = match spawn(engine, &capsule, &outcome.engine_log) {
        Ok(child) => child,
        Err(error) => {
            outcome.started = Some(Check::fail(error));
            return outcome;
        }
    };
    let launched = Instant::now();
    // SAFETY: getuid has no preconditions.
    let uid = unsafe { libc::getuid() };
    let socket = isonomic_proto::locate::engine_socket(&capsule, uid);
    let mut options = Options::new("cargo xtask report");
    options.reconnect_min = Duration::from_millis(50);
    options.reconnect_max = Duration::from_millis(250);
    let client = Client::start(
        &tokio::runtime::Handle::current(),
        unix_connector(&socket),
        options,
    );
    let mut live = client.live();

    // 1. Healthy.
    let healthy = wait_for(&mut live, &mut child, START_TIMEOUT, |live| {
        aggregate(live) == Some(ServiceState::Healthy)
    })
    .await;
    outcome.started = Some(match healthy {
        Ok(snapshot) => {
            outcome.healthy_after = Some(launched.elapsed());
            outcome.engine_version = snapshot.engine.clone();
            outcome.cold_start_ms = snapshot.system.as_ref().and_then(|s| s.cold_start_ms);
            // The engine records its cold start just after the services
            // report healthy, so the first healthy snapshot can lack it.
            let asked = Instant::now();
            while outcome.cold_start_ms.is_none() && asked.elapsed() < REAP_GRACE {
                outcome.cold_start_ms = client.status().await.ok().and_then(|s| s.cold_start_ms);
                if outcome.cold_start_ms.is_none() {
                    tokio::time::sleep(TICK).await;
                }
            }
            if let Ok(profile) = client.profile().await {
                outcome.profile = Some(profile.profile);
                outcome.memory_bytes = Some(profile.total_memory_bytes);
            }
            Check::pass(format!(
                "healthy {:.1} s after launch",
                launched.elapsed().as_secs_f64()
            ))
        }
        Err(why) => Check::fail(why),
    });

    // 2. Force-kill PostgreSQL; it must come back on its own.
    if outcome.started.as_ref().is_some_and(|c| c.passed) {
        outcome.recovered = Some(recover(&capsule, &mut live, &mut child).await);
    }

    // 3. Ordered shutdown, nothing left. Runs even after a failure, to clean up.
    outcome.stopped = Some(stop(&client, &capsule, &socket, &mut child, &outcome).await);
    outcome
}

fn spawn(engine: &Engine, capsule: &Path, log: &Path) -> Result<Child, String> {
    let out = File::create(log).map_err(|e| format!("creating {}: {e}", log.display()))?;
    let err = out.try_clone().map_err(|e| e.to_string())?;
    let mut command = Command::new(&engine.program);
    command.env_clear();
    for key in ["PATH", "HOME", "TMPDIR", "USER", "LOGNAME"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command.env("ISONOMIC_CAPSULE_DIR", capsule);
    if let Some(dir) = &engine.bundle_dir {
        command.env("ISONOMIC_BUNDLE_DIR", dir);
    }
    command
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", engine.program.display()))
}

async fn recover(capsule: &Path, live: &mut watch::Receiver<Live>, child: &mut Child) -> Check {
    let Some(pid) = postmaster_pid(capsule) else {
        return Check::fail("no postmaster.pid while healthy");
    };
    let before = postgres(&live.borrow()).map_or(0, |s| s.restart_count);
    let workers = children_of(pid);
    // SAFETY: plain signal to a process we just read from its pid file.
    unsafe { libc::kill(pid, libc::SIGKILL) };
    let killed = Instant::now();
    let back = wait_for(live, child, RECOVERY_TIMEOUT, |live| {
        postgres(live).is_some_and(|s| s.restart_count > before && s.state == ServiceState::Healthy)
    })
    .await;
    let snapshot = match back {
        Ok(snapshot) => snapshot,
        Err(why) => return Check::fail(format!("killed pid {pid}; {why}")),
    };
    let took = killed.elapsed();
    let restarts = postgres(&snapshot).map_or(0, |s| s.restart_count);
    let orphans = still_alive(&workers).await;
    let new_pid = postmaster_pid(capsule).unwrap_or(0);
    let detail = format!(
        "killed pid {pid} → healthy again in {:.1} s as pid {new_pid} · restarts {restarts} · \
         {} of {} old workers left",
        took.as_secs_f64(),
        orphans.len(),
        workers.len()
    );
    if new_pid == pid || !orphans.is_empty() {
        Check::fail(detail)
    } else {
        Check::pass(detail)
    }
}

async fn stop(
    client: &Client,
    capsule: &Path,
    socket: &Path,
    child: &mut Child,
    outcome: &Outcome,
) -> Check {
    let mut watched: Vec<i32> = Vec::new();
    if let Some(pid) = postmaster_pid(capsule) {
        watched.push(pid);
        watched.extend(children_of(pid));
    }
    let asked = Instant::now();
    let request = client.shutdown().await;
    let exit = wait_exit(child, SHUTDOWN_TIMEOUT).await;
    let took = asked.elapsed();
    let mut problems = Vec::new();
    if let Err(error) = request {
        problems.push(format!("system.shutdown failed: {error}"));
    }
    match exit {
        Some(status) if status.success() => {}
        Some(status) => problems.push(format!("engine exited with {status}")),
        None => {
            problems.push(format!(
                "engine still running after {}s",
                SHUTDOWN_TIMEOUT.as_secs()
            ));
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    let left = still_alive(&watched).await;
    for pid in &left {
        // SAFETY: cleaning up processes this run started.
        unsafe { libc::kill(*pid, libc::SIGKILL) };
    }
    if !left.is_empty() {
        problems.push(format!("still running: {left:?}"));
    }
    if postmaster_pid(capsule).is_some() {
        problems.push("postmaster.pid left behind (PostgreSQL did not stop cleanly)".into());
    }
    if socket.exists() {
        problems.push("engine socket left behind".into());
    }
    let engine_log = crate::util::tail(&outcome.engine_log, 400);
    let requested = engine_log
        .iter()
        .position(|l| l.contains("shutdown requested over the socket"));
    let last_is_stopped = engine_log.last().is_some_and(|l| l.ends_with("stopped"));
    let pg_down = std::fs::read_to_string(&outcome.postgres_log)
        .map(|log| log.contains("database system is shut down"))
        .unwrap_or(false);
    if requested.is_none() || !last_is_stopped {
        problems.push("engine log does not show request → stopped".into());
    }
    if !pg_down {
        problems.push("PostgreSQL log does not show a completed shutdown".into());
    }
    if problems.is_empty() {
        Check::pass(format!(
            "stopped in {:.1} s: PostgreSQL shut down cleanly, then the engine; socket removed; \
             0 of {} processes left",
            took.as_secs_f64(),
            watched.len() + 1
        ))
    } else {
        Check::fail(problems.join("; "))
    }
}

/// Wait until `done` holds, the engine exits, or time runs out.
async fn wait_for(
    live: &mut watch::Receiver<Live>,
    child: &mut Child,
    timeout: Duration,
    done: impl Fn(&Live) -> bool,
) -> Result<Live, String> {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let snapshot = live.borrow_and_update();
            if done(&snapshot) {
                return Ok(snapshot.clone());
            }
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("the engine exited early ({status})"));
        }
        if Instant::now() >= deadline {
            let state = describe(&live.borrow());
            return Err(format!(
                "timed out after {}s; last seen: {state}",
                timeout.as_secs()
            ));
        }
        tokio::select! {
            changed = live.changed() => {
                if changed.is_err() {
                    return Err("the client stopped".to_string());
                }
            }
            _ = tokio::time::sleep(TICK * 5) => {}
        }
    }
}

async fn wait_exit(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        tokio::time::sleep(TICK).await;
    }
}

fn aggregate(live: &Live) -> Option<ServiceState> {
    live.connected
        .then(|| live.system.as_ref().map(|s| s.aggregate))
        .flatten()
}

fn postgres(live: &Live) -> Option<&ServiceStatus> {
    live.system
        .as_ref()
        .and_then(|s| s.services.iter().find(|s| s.name == POSTGRES))
}

fn describe(live: &Live) -> String {
    if !live.connected {
        return "not connected".to_string();
    }
    match postgres(live) {
        Some(s) => format!("{} {:?}, {} restarts", s.name, s.state, s.restart_count),
        None => "connected, no PostgreSQL status yet".to_string(),
    }
}

/// The running postmaster, from its pid file (gone after a clean stop).
fn postmaster_pid(capsule: &Path) -> Option<i32> {
    let file = capsule
        .join("services")
        .join(POSTGRES)
        .join("postmaster.pid");
    let text = std::fs::read_to_string(file).ok()?;
    text.lines().next()?.trim().parse().ok()
}

fn children_of(pid: i32) -> Vec<i32> {
    crate::util::output("pgrep", &["-P", &pid.to_string()], None)
        .map(|out| out.lines().filter_map(|l| l.trim().parse().ok()).collect())
        .unwrap_or_default()
}

/// Running, as opposed to gone or exited-but-not-yet-reaped (a zombie,
/// which a container's first process may never reap).
fn alive(pid: i32) -> bool {
    match crate::util::output("ps", &["-o", "stat=", "-p", &pid.to_string()], None) {
        Some(stat) => !stat.is_empty() && !stat.starts_with('Z'),
        None => false,
    }
}

/// Which of `pids` are still running once the grace period is over.
async fn still_alive(pids: &[i32]) -> Vec<i32> {
    let deadline = Instant::now() + REAP_GRACE;
    loop {
        let left: Vec<i32> = pids.iter().copied().filter(|p| alive(*p)).collect();
        if left.is_empty() || Instant::now() >= deadline {
            return left;
        }
        tokio::time::sleep(TICK).await;
    }
}

// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Milestone 1 gate, driven through the SDK client exactly as the window
//! drives the engine: start until every core service is healthy, force-kill
//! each one and watch it come back, stop in order with nothing left, then
//! start again on the same Capsule and time a warm start.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use isonomic_client::{unix_connector, Client, Live, Options};
use isonomic_proto::system::{ServiceState, ServiceStatus};
use tokio::sync::watch;

use crate::procs::{self, REAP_GRACE, TICK};

/// First start includes first-time database setup and, on macOS, the
/// system's one-off check of programs it has not run before.
const START_TIMEOUT: Duration = Duration::from_secs(180);
const RECOVERY_TIMEOUT: Duration = Duration::from_secs(90);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60);
/// The plan's gate: every core service healthy within this of launch.
pub const WARM_TARGET: Duration = Duration::from_secs(10);
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
    pub fn pass(detail: impl Into<String>) -> Self {
        Self {
            passed: true,
            detail: detail.into(),
        }
    }
    pub fn fail(detail: impl Into<String>) -> Self {
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
    /// One per core service, in the engine's order.
    pub recovered: Vec<(String, Check)>,
    pub stopped: Option<Check>,
    pub warm: Option<Check>,
    pub services: Vec<String>,
    pub engine_version: Option<String>,
    pub cold_start_ms: Option<u64>,
    pub healthy_after: Option<Duration>,
    pub profile: Option<String>,
    pub memory_bytes: Option<u64>,
    /// The engine's standard error: what it said before it accepted the
    /// Capsule.
    pub engine_log: PathBuf,
    pub postgres_log: PathBuf,
}

/// The engine's last lines: its standard error at `stderr`, then its own
/// log in the Capsule (`logs/engine.log`), where it writes once it has
/// accepted the Capsule.
pub fn engine_lines(stderr: &Path, capsule: &Path, count: usize) -> Vec<String> {
    let mut lines = crate::util::tail(stderr, count);
    lines.extend(crate::util::tail(
        &capsule.join("logs").join("engine.log"),
        count,
    ));
    let skip = lines.len().saturating_sub(count);
    lines.split_off(skip)
}

impl Outcome {
    pub fn passed(&self) -> bool {
        let ok = |c: &Option<Check>| c.as_ref().is_some_and(|c| c.passed);
        ok(&self.started)
            && !self.recovered.is_empty()
            && self.recovered.iter().all(|(_, c)| c.passed)
            && ok(&self.stopped)
            && ok(&self.warm)
    }
}

/// One running engine and the client watching it.
struct Session {
    child: Child,
    client: Client,
    live: watch::Receiver<Live>,
    socket: PathBuf,
    launched: Instant,
    log: PathBuf,
}

impl Session {
    fn launch(engine: &Engine, capsule: &Path, log: &Path) -> Result<Self, String> {
        let child = spawn(engine, capsule, log)?;
        let launched = Instant::now();
        // SAFETY: getuid has no preconditions.
        let uid = unsafe { libc::getuid() };
        let socket = isonomic_proto::locate::engine_socket(capsule, uid);
        let mut options = Options::new("cargo xtask report");
        options.reconnect_min = Duration::from_millis(50);
        options.reconnect_max = Duration::from_millis(250);
        let client = Client::start(
            &tokio::runtime::Handle::current(),
            unix_connector(&socket),
            options,
        );
        let live = client.live();
        Ok(Self {
            child,
            client,
            live,
            socket,
            launched,
            log: log.to_path_buf(),
        })
    }

    /// The run directory: where the socket and the services' pid files are.
    fn run_dir(&self) -> PathBuf {
        self.socket
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    async fn healthy(&mut self) -> Result<Live, String> {
        wait_for(&mut self.live, &mut self.child, START_TIMEOUT, |live| {
            aggregate(live) == Some(ServiceState::Healthy)
        })
        .await
    }
}

pub async fn run(engine: &Engine, scratch: &Path) -> Outcome {
    let capsule = scratch.join("capsule");
    let mut outcome = Outcome {
        engine_log: scratch.join("engine.log"),
        postgres_log: capsule.join("logs").join(format!("{POSTGRES}.log")),
        ..Outcome::default()
    };
    let mut session = match Session::launch(engine, &capsule, &outcome.engine_log) {
        Ok(session) => session,
        Err(error) => {
            outcome.started = Some(Check::fail(error));
            return outcome;
        }
    };

    // 1. Every core service healthy.
    outcome.started = Some(match session.healthy().await {
        Ok(snapshot) => {
            outcome.healthy_after = Some(session.launched.elapsed());
            outcome.engine_version = snapshot.engine.clone();
            outcome.services = core_services(&snapshot);
            outcome.cold_start_ms = cold_start(&session.client, &snapshot).await;
            if let Ok(profile) = session.client.profile().await {
                outcome.profile = Some(profile.profile);
                outcome.memory_bytes = Some(profile.total_memory_bytes);
            }
            Check::pass(format!(
                "{} healthy {:.1} s after the first launch (first-time setup included)",
                outcome.services.join(" and "),
                session.launched.elapsed().as_secs_f64()
            ))
        }
        Err(why) => Check::fail(why),
    });

    // 2. Force-kill each core service; each must come back on its own.
    if outcome.started.as_ref().is_some_and(|c| c.passed) {
        for name in outcome.services.clone() {
            let check = recover(&mut session, &name).await;
            outcome.recovered.push((name, check));
        }
    }

    // 3. Ordered stop, nothing left. Runs even after a failure, to clean up.
    let stopped = stop(&mut session, &capsule, &outcome).await;
    let clean = stopped.passed;
    outcome.stopped = Some(stopped);

    // 4. Warm start on the same Capsule: the plan's ten-second gate.
    if clean {
        outcome.warm = Some(warm(engine, &capsule, scratch, &outcome).await);
    }
    outcome
}

async fn warm(engine: &Engine, capsule: &Path, scratch: &Path, outcome: &Outcome) -> Check {
    let log = scratch.join("engine-warm.log");
    let mut session = match Session::launch(engine, capsule, &log) {
        Ok(session) => session,
        Err(error) => return Check::fail(error),
    };
    let result = session.healthy().await;
    let took = session.launched.elapsed();
    let warm_outcome = Outcome {
        engine_log: log,
        postgres_log: outcome.postgres_log.clone(),
        services: outcome.services.clone(),
        ..Outcome::default()
    };
    let stopped = stop(&mut session, capsule, &warm_outcome).await;
    match result {
        Err(why) => Check::fail(why),
        Ok(_) if !stopped.passed => Check::fail(format!(
            "healthy {:.1} s after launch, but the second stop failed: {}",
            took.as_secs_f64(),
            stopped.detail
        )),
        Ok(_) => {
            let detail = format!(
                "every core service healthy {:.1} s after launch (target under {} s); stopped clean again",
                took.as_secs_f64(),
                WARM_TARGET.as_secs()
            );
            if took < WARM_TARGET {
                Check::pass(detail)
            } else {
                Check::fail(detail)
            }
        }
    }
}

/// The engine records its cold start just after the services report
/// healthy, so the first healthy snapshot can lack it.
async fn cold_start(client: &Client, snapshot: &Live) -> Option<u64> {
    let mut found = snapshot.system.as_ref().and_then(|s| s.cold_start_ms);
    let asked = Instant::now();
    while found.is_none() && asked.elapsed() < REAP_GRACE {
        found = client.status().await.ok().and_then(|s| s.cold_start_ms);
        if found.is_none() {
            tokio::time::sleep(TICK).await;
        }
    }
    found
}

fn core_services(live: &Live) -> Vec<String> {
    live.system
        .as_ref()
        .map(|s| {
            s.services
                .iter()
                .filter(|s| s.tier == 1)
                .map(|s| s.name.clone())
                .collect()
        })
        .unwrap_or_default()
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

async fn recover(session: &mut Session, name: &str) -> Check {
    let run_dir = session.run_dir();
    let Some(pid) = procs::service_pid(&run_dir, name) else {
        return Check::fail(format!("no pid file for {name} while healthy"));
    };
    let before = status(&session.live.borrow(), name).map_or(0, |s| s.restart_count);
    let workers = procs::children_of(pid);
    // SAFETY: plain signal to a process the engine noted as this service.
    unsafe { libc::kill(pid, libc::SIGKILL) };
    let killed = Instant::now();
    let back = wait_for(
        &mut session.live,
        &mut session.child,
        RECOVERY_TIMEOUT,
        |live| {
            status(live, name)
                .is_some_and(|s| s.restart_count > before && s.state == ServiceState::Healthy)
        },
    )
    .await;
    let snapshot = match back {
        Ok(snapshot) => snapshot,
        Err(why) => return Check::fail(format!("killed pid {pid}; {why}")),
    };
    let took = killed.elapsed();
    let restarts = status(&snapshot, name).map_or(0, |s| s.restart_count);
    let orphans = procs::still_alive(&workers).await;
    let new_pid = procs::service_pid(&run_dir, name).unwrap_or(0);
    let detail = format!(
        "killed pid {pid} → healthy again in {:.1} s as pid {new_pid} · restarts {restarts} · \
         {} of {} old workers left",
        took.as_secs_f64(),
        orphans.len(),
        workers.len()
    );
    if new_pid == pid || new_pid == 0 || !orphans.is_empty() {
        Check::fail(detail)
    } else {
        Check::pass(detail)
    }
}

async fn stop(session: &mut Session, capsule: &Path, outcome: &Outcome) -> Check {
    let run_dir = session.run_dir();
    let watched = procs::everything(&run_dir, &outcome.services);
    let asked = Instant::now();
    let request = session.client.shutdown().await;
    let exit = wait_exit(&mut session.child, SHUTDOWN_TIMEOUT).await;
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
            let _ = session.child.kill();
            let _ = session.child.wait();
        }
    }
    let left = procs::still_alive(&watched).await;
    for pid in &left {
        // SAFETY: cleaning up processes this run started.
        unsafe { libc::kill(*pid, libc::SIGKILL) };
    }
    if !left.is_empty() {
        problems.push(format!("still running: {left:?}"));
    }
    if procs::postmaster_pid(capsule).is_some() {
        problems.push("postmaster.pid left behind (PostgreSQL did not stop cleanly)".into());
    }
    let records: Vec<&String> = outcome
        .services
        .iter()
        .filter(|name| procs::service_pid(&run_dir, name).is_some())
        .collect();
    if !records.is_empty() {
        problems.push(format!("pid files left behind for {records:?}"));
    }
    if session.socket.exists() {
        problems.push("engine socket left behind".into());
    }
    let engine_log = engine_lines(&session.log, capsule, 400);
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
            "stopped in {:.1} s: every service, then the engine; socket and pid files removed; \
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

fn status<'a>(live: &'a Live, name: &str) -> Option<&'a ServiceStatus> {
    live.system
        .as_ref()
        .and_then(|s| s.services.iter().find(|s| s.name == name))
}

fn describe(live: &Live) -> String {
    if !live.connected {
        return "not connected".to_string();
    }
    match &live.system {
        Some(system) => system
            .services
            .iter()
            .map(|s| format!("{} {:?}, {} restarts", s.name, s.state, s.restart_count))
            .collect::<Vec<_>>()
            .join("; "),
        None => "connected, no status yet".to_string(),
    }
}

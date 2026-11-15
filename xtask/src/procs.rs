//! Process facts the gate checks from outside the engine: which pid runs a
//! service, its workers, and whether they are gone.

use std::path::Path;
use std::time::{Duration, Instant};

/// How long processes may take to disappear after their parent has gone.
pub const REAP_GRACE: Duration = Duration::from_secs(5);
pub const TICK: Duration = Duration::from_millis(50);

/// The pid the engine noted for a service in its run directory (the
/// directory that holds the engine's socket). Gone after a clean stop.
pub fn service_pid(run_dir: &Path, service: &str) -> Option<i32> {
    let text = std::fs::read_to_string(run_dir.join(format!("{service}.pid"))).ok()?;
    text.lines().next()?.trim().parse().ok()
}

/// PostgreSQL's own pid file: present while a server owns the data
/// directory, removed by a clean shutdown.
pub fn postmaster_pid(capsule: &Path) -> Option<i32> {
    let file = capsule
        .join("services")
        .join("postgres")
        .join("postmaster.pid");
    let text = std::fs::read_to_string(file).ok()?;
    text.lines().next()?.trim().parse().ok()
}

pub fn children_of(pid: i32) -> Vec<i32> {
    crate::util::output("pgrep", &["-P", &pid.to_string()], None)
        .map(|out| out.lines().filter_map(|l| l.trim().parse().ok()).collect())
        .unwrap_or_default()
}

/// Running, as opposed to gone or exited-but-not-yet-reaped (a zombie,
/// which a container's first process may never reap).
pub fn alive(pid: i32) -> bool {
    match crate::util::output("ps", &["-o", "stat=", "-p", &pid.to_string()], None) {
        Some(stat) => !stat.is_empty() && !stat.starts_with('Z'),
        None => false,
    }
}

/// Which of `pids` are still running once the grace period is over.
pub async fn still_alive(pids: &[i32]) -> Vec<i32> {
    let deadline = Instant::now() + REAP_GRACE;
    loop {
        let left: Vec<i32> = pids.iter().copied().filter(|p| alive(*p)).collect();
        if left.is_empty() || Instant::now() >= deadline {
            return left;
        }
        tokio::time::sleep(TICK).await;
    }
}

/// Every service's pid and its workers, for the "nothing left" check.
pub fn everything(run_dir: &Path, services: &[String]) -> Vec<i32> {
    let mut pids = Vec::new();
    for name in services {
        if let Some(pid) = service_pid(run_dir, name) {
            pids.push(pid);
            pids.extend(children_of(pid));
        }
    }
    pids
}

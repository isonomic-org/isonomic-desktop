//! `cargo xtask report`: build, run the Milestone 1 gate against a scratch
//! Capsule, and print one block to paste back.
//!
//! On macOS the engine under test is the one inside `dist/Isonomic.app`, so
//! the report also proves the app finds its own PostgreSQL. Elsewhere it is
//! the engine repo's release build with its fetched `sidecars/`.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::bundle::{self, Built};
use crate::gate::{self, Check, Engine, Outcome};
use crate::util::{self, megabytes};

pub struct Options {
    pub host_only: bool,
    pub build: bool,
}

/// Engine log lines worth repeating in the report.
const ENGINE_FACTS: [&str; 4] = ["binary ", "hardware profile:", "postgres: ", "cold start:"];
/// Marks a gate item that doesn't apply to this milestone.
const NOT_APPLICABLE: &str = "n/a: ";

/// Item 3: nothing in Milestone 1 starts lazily.
fn lazy_row(item: &mut impl FnMut(String, Option<&Check>)) {
    let check = Check::pass(format!(
        "{NOT_APPLICABLE}no lazy service ships in Milestone 1 (the vector database waits for \
         wave 3); the engine's tests cover the lazy tier"
    ));
    item("3 the lazy service starts on use".into(), Some(&check));
}

/// Start at login can't be scripted from here: these are the steps to do by
/// hand, printed with every report.
const LOGIN_STEPS: [&str; 4] = [
    "open the app once from dist/ (installs the agent; Start at Login is checked in the menu bar)",
    "log out and back in: no window opens; open the app: one engine, all services healthy",
    "kill -9 the engine: the window shows it reconnecting, then healthy",
    "Quit from the menu: nothing runs afterwards and nothing restarts it",
];
const TAIL_LINES: usize = 25;

pub fn run(options: &Options) -> Result<(), String> {
    let (engine, app) = prepare(options)?;
    let scratch = std::env::temp_dir().join(format!("isonomic-report-{}", std::process::id()));
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let outcome = runtime.block_on(gate::run(&engine, &scratch));
    drop(runtime);

    let block = render(&engine, app.as_ref(), &outcome);
    print!("{block}");
    let saved = util::desktop_root().join("dist").join("report.txt");
    if std::fs::create_dir_all(saved.parent().expect("dist has a parent")).is_ok()
        && std::fs::write(&saved, &block).is_ok()
    {
        println!("saved to {}", saved.display());
    }
    let measured = app.is_some() || !cfg!(target_os = "macos");
    if outcome.passed() && measured {
        let _ = std::fs::remove_dir_all(&scratch);
        Ok(())
    } else {
        Err(format!(
            "gate failed; the scratch Capsule and logs are kept at {}",
            scratch.display()
        ))
    }
}

fn prepare(options: &Options) -> Result<(Engine, Option<Built>), String> {
    if cfg!(target_os = "macos") {
        let built = if options.build {
            bundle::run(&bundle::Options {
                host_only: options.host_only,
                example_app: false,
                sign_dry_run: false,
                check_build: false,
            })?
        } else {
            Built::existing(bundle::app_path())?
        };
        let engine = Engine {
            program: built.app.join("Contents/MacOS").join(bundle::ENGINE_BIN),
            bundle_dir: None,
        };
        return Ok((engine, Some(built)));
    }
    let capsule = util::capsule_root()?;
    if options.build {
        util::run(
            util::cargo(&capsule).args(["build", "--release", "--package", bundle::ENGINE_BIN]),
            "building the engine",
        )?;
        util::run(
            util::cargo(&capsule).args(["xtask", "fetch-sidecars"]),
            "fetching PostgreSQL (cargo xtask fetch-sidecars)",
        )?;
    }
    let engine = Engine {
        program: util::target_dir(&capsule)
            .join("release")
            .join(bundle::ENGINE_BIN),
        bundle_dir: Some(capsule.join("sidecars")),
    };
    Ok((engine, None))
}

fn render(engine: &Engine, app: Option<&Built>, outcome: &Outcome) -> String {
    let desktop = util::desktop_root();
    let capsule = util::sibling("isonomic-capsule");
    let sdk = util::sibling("isonomic-sdk");
    let ui = util::sibling("isonomic-ui");
    // The database log sits in `<capsule>/logs/`, beside the engine's own.
    let run_capsule = outcome.postgres_log.parent().and_then(Path::parent);
    let engine_log = match run_capsule {
        Some(capsule) => crate::gate::engine_lines(&outcome.engine_log, capsule, 400),
        None => util::tail(&outcome.engine_log, 400),
    };
    let mut out = Vec::new();
    let mut row = |label: &str, value: String| out.push(format!("{label:<11} {value}"));

    row(
        "report",
        format!("Isonomic Desktop, Milestone 1 · {}", utc_now()),
    );
    row("machine", machine(outcome));
    row(
        "toolchain",
        util::output("rustc", &["-V"], Some(&desktop)).unwrap_or_else(|| "?".into()),
    );
    row(
        "window",
        format!(
            "{} · isonomic-desktop {}",
            env!("CARGO_PKG_VERSION"),
            util::git_rev(&desktop)
        ),
    );
    row(
        "engine",
        format!(
            "{} · isonomic-capsule {}",
            outcome.engine_version.as_deref().unwrap_or("?"),
            util::git_rev(&capsule)
        ),
    );
    row(
        "sdk",
        format!(
            "isonomic-sdk {} · SDK_REF window {} · engine {}",
            util::git_rev(&sdk),
            pin(&desktop, "SDK_REF"),
            pin(&capsule, "SDK_REF")
        ),
    );
    row(
        "ui",
        format!(
            "isonomic-ui {} · UI_REF (the SDK's) {}",
            util::git_rev(&ui),
            pin(&sdk, "UI_REF")
        ),
    );
    row("postgresql", postgres_version(&engine_log));
    match app {
        Some(built) => {
            row(
                "app",
                format!(
                    "{} · {} · {} · ad-hoc signed",
                    built.app.display(),
                    built.architectures(),
                    megabytes(built.bytes)
                ),
            );
            for (part, bytes) in &built.parts {
                row("", format!("  {part:<52} {:>9}", megabytes(*bytes)));
            }
        }
        None => row("app", "not built: the app bundle is macOS only".into()),
    }
    row("engine bin", engine.program.display().to_string());
    let mut first = true;
    for line in engine_log
        .iter()
        .filter(|l| ENGINE_FACTS.iter().any(|f| fact(l, f)))
    {
        let label = if first { "engine log" } else { "" };
        first = false;
        row(label, strip_prefix(line).to_string());
    }
    row(
        "cold start",
        match (outcome.cold_start_ms, outcome.healthy_after) {
            (Some(ms), Some(after)) => format!(
                "{ms} ms (engine: core services up) · healthy {:.1} s after launch",
                after.as_secs_f64()
            ),
            _ => "not reached".into(),
        },
    );

    out.push("gate".to_string());
    let measured = Check {
        passed: outcome.cold_start_ms.is_some() && app.is_some(),
        detail: match app {
            Some(b) => format!("cold start above; app {}", megabytes(b.bytes)),
            None => "no app bundle on this platform".into(),
        },
    };
    let on_mac = cfg!(target_os = "macos");
    let mut item = |name: String, check: Option<&Check>| {
        let (verdict, detail) = match check {
            Some(check) if check.detail.starts_with(NOT_APPLICABLE) => {
                ("n/a ", &check.detail[NOT_APPLICABLE.len()..])
            }
            None => ("skip", "not run"),
            Some(check) if check.passed => ("PASS", check.detail.as_str()),
            Some(check) => ("FAIL", check.detail.as_str()),
        };
        out.push(format!("  {name:<40} {verdict}  {detail}"));
    };
    item(
        "1 every core service healthy".into(),
        outcome.started.as_ref(),
    );
    if outcome.recovered.is_empty() {
        item("2 each core service killed, comes back".into(), None);
    }
    for (name, check) in &outcome.recovered {
        item(format!("2 force-killed {name} comes back"), Some(check));
    }
    lazy_row(&mut item);
    item(
        "4 ordered stop, nothing left running".into(),
        outcome.stopped.as_ref(),
    );
    item(
        format!("5 warm start under {} s", gate::WARM_TARGET.as_secs()),
        outcome.warm.as_ref(),
    );
    let size = if on_mac {
        if measured.passed {
            "PASS"
        } else {
            "FAIL"
        }
    } else {
        "n/a "
    };
    out.push(format!(
        "  {:<40} {size}  {}",
        "6 cold start and bundle size reported", measured.detail
    ));
    out.push("  by hand: start at login".into());
    for (index, step) in LOGIN_STEPS.iter().enumerate() {
        out.push(format!("    {}. {step}", index + 1));
    }
    let result = match (outcome.passed(), measured.passed || !on_mac) {
        (true, true) if on_mac => "PASS (start at login: by hand, above)",
        (true, true) => "PASS for items 1-5; item 6 needs the Mac",
        _ => "FAIL",
    };
    out.push(format!("result      {result}"));

    if !outcome.passed() {
        let warm_log = outcome.engine_log.with_file_name("engine-warm.log");
        let cache_log = outcome.postgres_log.with_file_name("cache.log");
        let own_log = outcome.postgres_log.with_file_name("engine.log");
        for (name, path) in [
            ("engine, standard error", &outcome.engine_log),
            ("engine, warm start, standard error", &warm_log),
            ("engine's own log", &own_log),
            ("postgres", &outcome.postgres_log),
            ("cache", &cache_log),
        ] {
            if !path.exists() {
                continue;
            }
            out.push(format!(
                "--- last {TAIL_LINES} lines: {name} ({})",
                path.display()
            ));
            out.extend(util::tail(path, TAIL_LINES));
        }
    }
    let rule = "-".repeat(72);
    let block = format!("{rule}\n{}\n{rule}\n", out.join("\n"));
    // Paths under the home folder print as `~/...`, so a pasted block does
    // not carry the machine's user name.
    match std::env::var("HOME") {
        Ok(home) if home.len() > 1 => block.replace(&home, "~"),
        _ => block,
    }
}

fn fact(line: &str, prefix: &str) -> bool {
    strip_prefix(line).starts_with(prefix)
}

fn strip_prefix(line: &str) -> &str {
    line.strip_prefix("[isonomic-capsule] ").unwrap_or(line)
}

/// The short commit a repo pins a sibling to (`SDK_REF`, `UI_REF`).
fn pin(repo: &Path, file: &str) -> String {
    std::fs::read_to_string(repo.join(file))
        .map(|s| s.trim().chars().take(7).collect())
        .unwrap_or_else(|_| "?".into())
}

/// Ask the PostgreSQL the engine resolved for its version.
fn postgres_version(engine_log: &[String]) -> String {
    let path = engine_log.iter().find_map(|l| {
        let rest = strip_prefix(l).strip_prefix("binary postgres: ")?;
        Some(PathBuf::from(
            rest.rsplit_once(" (").map_or(rest, |(p, _)| p),
        ))
    });
    path.and_then(|p| util::output(&p.to_string_lossy(), &["--version"], None))
        .unwrap_or_else(|| "?".into())
}

fn machine(outcome: &Outcome) -> String {
    let mut parts = Vec::new();
    if cfg!(target_os = "macos") {
        let sysctl = |key: &str| util::output("sysctl", &["-n", key], None);
        parts.extend(sysctl("hw.model"));
        parts.extend(sysctl("machdep.cpu.brand_string"));
        if let Some(version) = util::output("sw_vers", &["-productVersion"], None) {
            parts.push(format!("macOS {version}"));
        }
    } else {
        let cpu = std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|info| {
                info.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.split_once(':'))
                    .map(|(_, v)| v.trim().to_string())
            });
        parts.extend(cpu);
        parts.extend(util::output("uname", &["-sr"], None));
    }
    parts.push(std::env::consts::ARCH.to_string());
    if let Some(bytes) = outcome.memory_bytes {
        parts.push(format!("{} GB", bytes / (1 << 30)));
    }
    if let Some(profile) = &outcome.profile {
        parts.push(format!("profile {profile}"));
    }
    parts.join(" · ")
}

/// `YYYY-MM-DD HH:MM UTC`, without a date library.
fn utc_now() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60
    )
}

/// Days since 1970-01-01 to a calendar date (proleptic Gregorian).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_406), (2025, 11, 14));
        assert_eq!(civil_from_days(20_771), (2026, 11, 14));
    }

    #[test]
    fn reads_the_resolved_postgres_path() {
        let lines = vec![
            "[isonomic-capsule] hardware profile: standard".to_string(),
            "[isonomic-capsule] binary postgres: /nowhere/bin/postgres (missing)".to_string(),
        ];
        assert!(fact(&lines[0], "hardware profile:"));
        // The path does not exist, so the version is unknown, not an error.
        assert_eq!(postgres_version(&lines), "?");
    }
}

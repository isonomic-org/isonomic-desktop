//! `cargo xtask bundle`: build `dist/Isonomic.app`.
//!
//! Layout (packaging spec):
//!
//! ```text
//! Isonomic.app/Contents/
//!   Info.plist
//!   MacOS/isonomic-desktop        the window, both architectures fused
//!   MacOS/isonomic-capsule        the engine, both architectures fused
//!   Resources/sidecars/postgres-x86_64-apple-darwin/{bin,lib,share,...}
//!   Resources/sidecars/postgres-aarch64-apple-darwin/{bin,lib,share,...}
//! ```
//!
//! Directory trees ship once per architecture; the running engine picks the
//! tree for its own slice. Everything is signed ad hoc.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::{self, megabytes};

pub const APP_NAME: &str = "Isonomic";
pub const BUNDLE_ID: &str = "com.isonomic.desktop";
pub const WINDOW_BIN: &str = "isonomic-desktop";
pub const ENGINE_BIN: &str = "isonomic-capsule";
/// The oldest macOS the app declares: 11, the first release that runs on
/// Apple Silicon. Nothing older has been tried.
const MINIMUM_MACOS: &str = "11.0";
const APPLE_SLICES: [&str; 2] = ["x86_64-apple-darwin", "aarch64-apple-darwin"];
/// What the app ships of each PostgreSQL tree. Headers (`include`) and the
/// upstream readme are left out: the engine never reads them.
const POSTGRES_PARTS: [&str; 5] = ["bin", "lib", "share", "COPYRIGHT", "LICENSE"];

pub struct Options {
    /// Build for this Mac's architecture only (faster; not universal).
    pub host_only: bool,
}

/// What `bundle` produced.
pub struct Built {
    pub app: PathBuf,
    pub targets: Vec<String>,
    pub bytes: u64,
    /// Size of each part, largest pieces of the bundle.
    pub parts: Vec<(String, u64)>,
}

impl Built {
    /// Measure an app that is already on disk (`report --no-build`).
    pub fn existing(app: PathBuf) -> Result<Self, String> {
        if !app.join("Contents/Info.plist").is_file() {
            return Err(format!(
                "no app at {}: run `cargo xtask bundle` first",
                app.display()
            ));
        }
        let sidecars = app.join("Contents/Resources/sidecars");
        let targets = APPLE_SLICES
            .iter()
            .filter(|t| sidecars.join(format!("postgres-{t}")).is_dir())
            .map(|t| t.to_string())
            .collect();
        Ok(Self::measure(app, targets))
    }

    fn measure(app: PathBuf, targets: Vec<String>) -> Self {
        let mut parts = Vec::new();
        for name in [WINDOW_BIN, ENGINE_BIN] {
            let file = app.join("Contents/MacOS").join(name);
            // What the file really holds, not what was asked for.
            let arches = util::output("lipo", &["-archs", &file.to_string_lossy()], None)
                .unwrap_or_else(|| "?".into());
            parts.push((format!("MacOS/{name} ({arches})"), util::dir_size(&file)));
        }
        for target in &targets {
            let tree = format!("Resources/sidecars/postgres-{target}");
            parts.push((
                tree.clone(),
                util::dir_size(&app.join("Contents").join(tree)),
            ));
        }
        let bytes = util::dir_size(&app);
        Self {
            app,
            targets,
            bytes,
            parts,
        }
    }

    pub fn architectures(&self) -> String {
        let arches: Vec<&str> = self.targets.iter().map(|t| arch_of(t)).collect();
        if arches.len() > 1 {
            format!("universal ({})", arches.join(", "))
        } else {
            arches.join(", ")
        }
    }

    pub fn print(&self) {
        println!(
            "built {} · {} · {}",
            self.app.display(),
            self.architectures(),
            megabytes(self.bytes)
        );
        for (name, bytes) in &self.parts {
            println!("  {name:<52} {:>9}", megabytes(*bytes));
        }
    }
}

fn arch_of(triple: &str) -> &str {
    triple.split('-').next().unwrap_or(triple)
}

/// This Mac's own target triple.
pub fn host_target() -> String {
    format!("{}-apple-darwin", std::env::consts::ARCH)
}

pub fn app_path() -> PathBuf {
    util::desktop_root()
        .join("dist")
        .join(format!("{APP_NAME}.app"))
}

pub fn run(options: &Options) -> Result<Built, String> {
    if !cfg!(target_os = "macos") {
        return Err("cargo xtask bundle builds a macOS app: run it on a Mac".to_string());
    }
    let desktop = util::desktop_root();
    let capsule = util::capsule_root()?;
    let targets: Vec<String> = if options.host_only {
        vec![host_target()]
    } else {
        APPLE_SLICES.iter().map(|t| t.to_string()).collect()
    };

    for target in &targets {
        build(&desktop, WINDOW_BIN, target)?;
        build(&capsule, ENGINE_BIN, target)?;
    }
    let mut fetch = util::cargo(&capsule);
    fetch.args(["xtask", "fetch-sidecars"]);
    if options.host_only {
        fetch.arg(format!("--target={}", targets[0]));
    }
    util::run(
        &mut fetch,
        "fetching PostgreSQL (cargo xtask fetch-sidecars)",
    )?;

    let app = app_path();
    assemble(&app, &targets, &desktop, &capsule)?;
    sign(&app)?;
    Ok(Built::measure(app, targets))
}

fn build(repo: &Path, package: &str, target: &str) -> Result<(), String> {
    println!("building {package} for {target}");
    util::run(
        util::cargo(repo).args([
            "build",
            "--release",
            "--package",
            package,
            "--target",
            target,
        ]),
        &format!("building {package} for {target}"),
    )
}

fn release_binary(repo: &Path, target: &str, name: &str) -> PathBuf {
    util::target_dir(repo)
        .join(target)
        .join("release")
        .join(name)
}

fn assemble(app: &Path, targets: &[String], desktop: &Path, capsule: &Path) -> Result<(), String> {
    if app.exists() {
        std::fs::remove_dir_all(app).map_err(|e| format!("removing old {}: {e}", app.display()))?;
    }
    let contents = app.join("Contents");
    let macos = contents.join("MacOS");
    let sidecars = contents.join("Resources/sidecars");
    for dir in [&macos, &sidecars] {
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }

    for (repo, name) in [(desktop, WINDOW_BIN), (capsule, ENGINE_BIN)] {
        let slices: Vec<PathBuf> = targets
            .iter()
            .map(|t| release_binary(repo, t, name))
            .collect();
        fuse(&slices, &macos.join(name))?;
    }

    for target in targets {
        let tree = format!("postgres-{target}");
        let source = capsule.join("sidecars").join(&tree);
        if !source.join("bin/postgres").is_file() {
            return Err(format!(
                "{} is missing: run `cargo xtask fetch-sidecars` in the engine repo",
                source.display()
            ));
        }
        for part in POSTGRES_PARTS {
            let from = source.join(part);
            if from.exists() {
                // ditto keeps symlinks (the shared libraries use them) and modes.
                let to = sidecars.join(&tree).join(part);
                util::run(
                    Command::new("ditto").arg(&from).arg(&to),
                    &format!("copying {}", from.display()),
                )?;
            }
        }
    }

    let plist = info_plist(env!("CARGO_PKG_VERSION"));
    std::fs::write(contents.join("Info.plist"), plist).map_err(|e| format!("Info.plist: {e}"))
}

/// One slice is copied; several are fused into one universal file.
fn fuse(slices: &[PathBuf], out: &Path) -> Result<(), String> {
    for slice in slices {
        if !slice.is_file() {
            return Err(format!("missing build output {}", slice.display()));
        }
    }
    if let [only] = slices {
        std::fs::copy(only, out).map_err(|e| format!("copying {}: {e}", only.display()))?;
        return Ok(());
    }
    util::run(
        Command::new("lipo")
            .arg("-create")
            .args(slices)
            .arg("-output")
            .arg(out),
        &format!("fusing {}", out.display()),
    )
}

pub fn info_plist(version: &str) -> String {
    let entries = [
        ("CFBundleDevelopmentRegion", "en"),
        ("CFBundleDisplayName", APP_NAME),
        ("CFBundleExecutable", WINDOW_BIN),
        ("CFBundleIdentifier", BUNDLE_ID),
        ("CFBundleInfoDictionaryVersion", "6.0"),
        ("CFBundleName", APP_NAME),
        ("CFBundlePackageType", "APPL"),
        ("CFBundleShortVersionString", version),
        ("CFBundleVersion", version),
        ("LSMinimumSystemVersion", MINIMUM_MACOS),
    ];
    let mut body = String::new();
    for (key, value) in entries {
        body.push_str(&format!("  <key>{key}</key>\n  <string>{value}</string>\n"));
    }
    body.push_str("  <key>NSHighResolutionCapable</key>\n  <true/>\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n{body}</dict>\n</plist>\n"
    )
}

/// Ad-hoc signing: every nested Mach-O file first, then the bundle, which
/// signs the main program last (codesign refuses a main program whose
/// nested code is still unsigned). Apple Silicon refuses to run unsigned
/// code, and an ad-hoc signature is enough on the machine that built it.
fn sign(app: &Path) -> Result<(), String> {
    let main = app.join("Contents/MacOS").join(WINDOW_BIN);
    let mut code = Vec::new();
    collect_mach_o(&app.join("Contents"), &mut code);
    code.retain(|file| *file != main);
    for chunk in code.chunks(200) {
        util::run_quiet(
            Command::new("codesign")
                .args(["--force", "--sign", "-", "--timestamp=none"])
                .args(chunk),
            "signing the bundled programs",
        )?;
    }
    util::run_quiet(
        Command::new("codesign")
            .args(["--force", "--sign", "-", "--timestamp=none"])
            .arg(app),
        "signing the app",
    )?;
    util::run_quiet(
        Command::new("codesign")
            .args(["--verify", "--strict"])
            .arg(app),
        "verifying the app signature",
    )
}

fn collect_mach_o(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            collect_mach_o(&path, out);
        } else if meta.is_file() && is_mach_o(&path) {
            out.push(path);
        }
    }
}

/// Thin (32 or 64 bit, either byte order) or fat Mach-O.
pub fn is_mach_o(path: &Path) -> bool {
    use std::io::Read;
    let mut magic = [0u8; 4];
    let read = std::fs::File::open(path).and_then(|mut f| f.read_exact(&mut magic));
    read.is_ok()
        && matches!(
            u32::from_be_bytes(magic),
            0xfeed_face | 0xfeed_facf | 0xcefa_edfe | 0xcffa_edfe | 0xcafe_babe | 0xbeba_feca
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_names_the_window_and_the_isonomic_id() {
        let plist = info_plist("1.2.3");
        assert!(plist
            .contains("<key>CFBundleIdentifier</key>\n  <string>com.isonomic.desktop</string>"));
        assert!(
            plist.contains("<key>CFBundleExecutable</key>\n  <string>isonomic-desktop</string>")
        );
        assert!(plist.contains("<string>1.2.3</string>"));
        assert!(plist.trim_end().ends_with("</plist>"));
    }

    #[test]
    fn recognises_mach_o_headers() {
        let dir = std::env::temp_dir().join(format!("xtask-macho-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let thin = dir.join("thin");
        let fat = dir.join("fat");
        let text = dir.join("text");
        std::fs::write(&thin, [0xcf, 0xfa, 0xed, 0xfe, 7, 0, 0, 1]).unwrap();
        std::fs::write(&fat, [0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 2]).unwrap();
        std::fs::write(&text, "#!/bin/sh\n").unwrap();
        assert!(is_mach_o(&thin));
        assert!(is_mach_o(&fat));
        assert!(!is_mach_o(&text));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_architectures() {
        let built = Built {
            app: PathBuf::from("/x.app"),
            targets: APPLE_SLICES.iter().map(|t| t.to_string()).collect(),
            bytes: 0,
            parts: Vec::new(),
        };
        assert_eq!(built.architectures(), "universal (x86_64, aarch64)");
    }
}

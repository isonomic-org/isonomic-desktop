// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! The oldest macOS each Mach-O file in the bundle runs on, read from its
//! load commands (`LC_BUILD_VERSION`, or `LC_VERSION_MIN_MACOSX` in older
//! files), so the app never declares a minimum its own parts don't meet.

use std::path::{Path, PathBuf};

const LC_VERSION_MIN_MACOSX: u32 = 0x24;
const LC_BUILD_VERSION: u32 = 0x32;
const MH_MAGIC: u32 = 0xfeed_face;
const MH_MAGIC_64: u32 = 0xfeed_facf;
const FAT_MAGIC: u32 = 0xcafe_babe;
const CPU_X86_64: u32 = 0x0100_0007;
const CPU_ARM64: u32 = 0x0100_000c;

/// A macOS version as `(major, minor)`.
pub type Version = (u32, u32);

pub fn parse_version(text: &str) -> Option<Version> {
    let mut parts = text.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().map_or(Some(0), |m| m.parse().ok())?;
    Some((major, minor))
}

fn le(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn be(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn decode(packed: u32) -> Version {
    (packed >> 16, (packed >> 8) & 0xff)
}

/// The minimum macOS of one thin Mach-O image starting at `at`.
fn thin(bytes: &[u8], at: usize) -> Option<Version> {
    let magic = le(bytes, at)?;
    let header = match magic {
        MH_MAGIC_64 => 32,
        MH_MAGIC => 28,
        _ => return None,
    };
    let commands = le(bytes, at + 16)?;
    let mut cursor = at + header;
    for _ in 0..commands {
        let (command, size) = (le(bytes, cursor)?, le(bytes, cursor + 4)? as usize);
        match command {
            LC_BUILD_VERSION => return le(bytes, cursor + 12).map(decode),
            LC_VERSION_MIN_MACOSX => return le(bytes, cursor + 8).map(decode),
            _ => {}
        }
        if size == 0 {
            return None;
        }
        cursor += size;
    }
    None
}

fn arch_name(cputype: u32) -> &'static str {
    match cputype {
        CPU_X86_64 => "x86_64",
        CPU_ARM64 => "arm64",
        _ => "other",
    }
}

/// Each slice of a Mach-O file with its architecture and minimum; empty
/// when it is not one, or names none.
pub fn slices(bytes: &[u8]) -> Vec<(&'static str, Version)> {
    if be(bytes, 0) == Some(FAT_MAGIC) {
        let count = be(bytes, 4).unwrap_or(0) as usize;
        return (0..count)
            .filter_map(|i| {
                let entry = 8 + i * 20;
                let cpu = be(bytes, entry)?;
                let offset = be(bytes, entry + 8)? as usize;
                Some((arch_name(cpu), thin(bytes, offset)?))
            })
            .collect();
    }
    match (le(bytes, 4), thin(bytes, 0)) {
        (Some(cpu), Some(version)) => vec![(arch_name(cpu), version)],
        _ => Vec::new(),
    }
}

/// The highest minimum across every slice of a Mach-O file, or `None` when
/// it is not one or names none.
#[cfg(test)]
fn minimum(bytes: &[u8]) -> Option<Version> {
    slices(bytes).into_iter().map(|(_, v)| v).max()
}

/// Every slice of every Mach-O file under `dir` whose minimum is above
/// `declared`, as (file, architecture, minimum).
pub fn above(dir: &Path, declared: Version) -> Vec<(PathBuf, &'static str, Version)> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file() {
                let bytes = std::fs::read(&path).unwrap_or_default();
                for (arch, version) in slices(&bytes) {
                    if version > declared {
                        found.push((path.clone(), arch, version));
                    }
                }
            }
        }
    }
    found.sort();
    found
}

/// One part of the app that needs a newer macOS, per architecture.
#[derive(Debug, PartialEq, Eq)]
pub struct Part {
    pub name: String,
    pub arch: &'static str,
    /// The highest minimum among its files.
    pub minimum: Version,
    pub files: usize,
}

/// The part a file under `root` belongs to: a sidecar's directory without
/// its target triple (`postgres`), or the program's own name.
fn part_of(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let name = match parts.iter().position(|p| p == "sidecars") {
        Some(at) if at + 1 < parts.len() => parts[at + 1].clone(),
        _ => parts.last().cloned().unwrap_or_default(),
    };
    ["-x86_64-apple-darwin", "-aarch64-apple-darwin"]
        .iter()
        .find_map(|triple| name.strip_suffix(triple))
        .map_or(name.clone(), str::to_string)
}

/// What [`above`] found, one line per part and architecture.
pub fn summarize(root: &Path, found: &[(PathBuf, &'static str, Version)]) -> Vec<Part> {
    let mut summary: Vec<Part> = Vec::new();
    for (file, arch, version) in found {
        let name = part_of(root, file);
        match summary
            .iter_mut()
            .find(|p| p.name == name && p.arch == *arch)
        {
            Some(part) => {
                part.minimum = part.minimum.max(*version);
                part.files += 1;
            }
            None => summary.push(Part {
                name,
                arch,
                minimum: *version,
                files: 1,
            }),
        }
    }
    summary.sort_by(|a, b| (&a.name, a.arch).cmp(&(&b.name, b.arch)));
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 64-bit Mach-O header with one LC_BUILD_VERSION naming `minos`.
    fn image(minos: Version) -> Vec<u8> {
        let mut bytes = Vec::new();
        for word in [MH_MAGIC_64, 0x0100_0007, 3, 2, 1, 24, 0, 0] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let packed = (minos.0 << 16) | (minos.1 << 8);
        for word in [LC_BUILD_VERSION, 24, 1, packed, packed, 0] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn a_thin_and_a_fat_file_name_their_minimum() {
        assert_eq!(minimum(&image((13, 3))), Some((13, 3)));
        assert_eq!(minimum(b"not a program"), None);
        // Two slices at 11.0 and 13.3: the higher one counts.
        let (a, b) = (image((11, 0)), image((13, 3)));
        let mut fat = Vec::new();
        for word in [FAT_MAGIC, 2] {
            fat.extend_from_slice(&word.to_be_bytes());
        }
        let first = 8 + 2 * 20;
        let second = first + a.len();
        for (offset, len) in [(first, a.len()), (second, b.len())] {
            for word in [0x0100_0007u32, 3, offset as u32, len as u32, 0] {
                fat.extend_from_slice(&word.to_be_bytes());
            }
        }
        fat.extend_from_slice(&a);
        fat.extend_from_slice(&b);
        assert_eq!(minimum(&fat), Some((13, 3)));
        assert_eq!(slices(&fat), vec![("x86_64", (11, 0)), ("x86_64", (13, 3))]);
    }

    #[test]
    fn the_summary_names_each_sidecar_and_architecture() {
        let root = Path::new("/app");
        let side = |rest: &str| root.join("Contents/Resources/sidecars").join(rest);
        let found = vec![
            (
                side("postgres-aarch64-apple-darwin/bin/initdb"),
                "arm64",
                (26, 0),
            ),
            (
                side("postgres-aarch64-apple-darwin/lib/libpq.dylib"),
                "arm64",
                (15, 0),
            ),
            (
                side("postgres-x86_64-apple-darwin/bin/postgres"),
                "x86_64",
                (26, 0),
            ),
            (side("cache/cache-server"), "arm64", (15, 0)),
            (side("cache/cache-server"), "x86_64", (15, 0)),
        ];
        let lines: Vec<_> = summarize(root, &found)
            .into_iter()
            .map(|p| (p.name, p.arch, p.minimum, p.files))
            .collect();
        assert_eq!(
            lines,
            vec![
                ("cache".to_string(), "arm64", (15, 0), 1),
                ("cache".to_string(), "x86_64", (15, 0), 1),
                ("postgres".to_string(), "arm64", (26, 0), 2),
                ("postgres".to_string(), "x86_64", (26, 0), 1),
            ]
        );
    }

    #[test]
    fn versions_read_and_compare() {
        assert_eq!(parse_version("13.3"), Some((13, 3)));
        assert_eq!(parse_version("11"), Some((11, 0)));
        assert!(parse_version("13.3").unwrap() > parse_version("11.0").unwrap());
    }

    #[test]
    fn files_above_the_declared_minimum_are_named() {
        let dir = std::env::temp_dir().join(format!("macho-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sidecars")).unwrap();
        std::fs::write(dir.join("old"), image((11, 0))).unwrap();
        std::fs::write(dir.join("sidecars/new"), image((13, 3))).unwrap();
        std::fs::write(dir.join("notes.txt"), b"text").unwrap();
        let found = above(&dir, (11, 0));
        assert_eq!(found, vec![(dir.join("sidecars/new"), "x86_64", (13, 3))]);
        assert!(above(&dir, (13, 3)).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

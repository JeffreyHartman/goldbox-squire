//! Finds the system emulator on PATH.
//!
//! ADR 0003: a discovered install runs the emulator the user keeps working,
//! not the one a publisher shipped years ago. Distros disagree on the binary
//! name: on Arch-likes `dosbox` is either the original or dosbox-staging (the
//! two packages conflict over the name), other setups install the longer
//! names, and dosbox-x is always `dosbox-x`.

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;

/// The names tried, in order of preference.
pub const NAMES: [&str; 3] = ["dosbox", "dosbox-staging", "dosbox-x"];

/// The first of the common emulator names that exists as an executable in
/// `path` (a PATH-style list of folders). Name order beats folder order:
/// `dosbox` anywhere wins over `dosbox-staging` anywhere.
pub fn find_on_path(path: &OsStr) -> Option<&'static str> {
    NAMES.into_iter().find(|name| {
        std::env::split_paths(path).any(|dir| {
            std::fs::metadata(dir.join(name))
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        })
    })
}

/// `find_on_path` over the real PATH.
pub fn find() -> Option<&'static str> {
    std::env::var_os("PATH").and_then(|path| find_on_path(&path))
}

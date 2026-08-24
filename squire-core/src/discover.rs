//! Finds game installs on disk by their shape.
//!
//! Discovery is structural, not a list of publishers (ADR 0001). An install is
//! a directory holding a conf file with an `[autoexec]` section containing a
//! `mount` line, plus a save folder, named per game, holding `CHRDAT*.SAV`
//! files. Which game the install holds comes from that folder's name, via the
//! registry. A layout this has never seen fails cleanly to the manual path.

use std::path::{Path, PathBuf};

use crate::games;

/// How deep below a search root the walk goes. Both real layouts sit one or
/// two levels down; four leaves room without crawling a whole disk.
const MAX_DEPTH: usize = 4;

/// The publisher whose launch script named the conf order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publisher {
    Gog,
    Steam,
}

/// One install found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredInstall {
    /// The registry id of the game the install holds.
    pub game_id: String,
    /// Who laid the install out, when a launch script said so.
    pub publisher: Option<Publisher>,
    /// The folder holding the conf files. The emulator must start here,
    /// because both publishers' autoexecs use relative mounts.
    pub root: PathBuf,
    /// The save folder, relative to `root`.
    pub saves: PathBuf,
    /// The conf files, relative to `root`, in launch order.
    pub confs: Vec<String>,
    /// An emulator binary shipped inside the install. The publisher shipped a
    /// build known to run their conf, so it wins over `dosbox` on PATH.
    pub emulator: Option<PathBuf>,
}

/// The fixed roots discovery always searches, filtered to the ones that exist.
pub fn default_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return vec![PathBuf::from("/opt")];
    };
    [
        home.join(".local/share/Steam/steamapps/common"),
        home.join(".steam/steam/steamapps/common"),
        home.join("GOG Games"),
        home.join("Games"),
        home.join("gog"),
        home.join("goldbox"),
        PathBuf::from("/opt"),
    ]
    .into_iter()
    .collect()
}

/// Searches the roots and returns every install found, in a stable order.
pub fn discover(roots: &[PathBuf]) -> Vec<DiscoveredInstall> {
    let games = games::games();
    let mut found = Vec::new();

    // Two roots can reach the same folder (`~/.steam/steam` is a symlink into
    // `~/.local/share/Steam`, and both are search roots). Resolving each root
    // to its real path lets the dedup below collapse them.
    let mut resolved: Vec<PathBuf> = roots
        .iter()
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .collect();
    resolved.sort();
    resolved.dedup();

    for root in &resolved {
        let mut dirs = Vec::new();
        collect_dirs(root, 0, &mut dirs);
        for dir in dirs {
            found.extend(examine(&dir, &games));
        }
    }

    found.sort_by(|a, b| a.root.cmp(&b.root).then(a.game_id.cmp(&b.game_id)));
    found.dedup();
    found
}

/// Every directory from `dir` down, at most `MAX_DEPTH` levels below the root.
fn collect_dirs(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    out.push(dir.to_path_buf());
    if depth == MAX_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // A symlinked directory could loop the walk back on itself, so only
        // real directories are entered. `file_type` does not follow links.
        let is_real_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_real_dir {
            collect_dirs(&entry.path(), depth + 1, out);
        }
    }
}

/// Reads one directory as a possible install root.
fn examine(dir: &Path, games: &[games::Game]) -> Vec<DiscoveredInstall> {
    let confs = conf_files(dir);
    if confs.is_empty() {
        return Vec::new();
    }
    // The conf holding [autoexec] with a mount is what makes this a launchable
    // install rather than a stray settings file.
    if !confs.iter().any(|(_, has_autoexec)| *has_autoexec) {
        return Vec::new();
    }

    let mut found = Vec::new();
    for game in games {
        let Some(saves) = find_save_folder(dir, &game.save_folder) else {
            continue;
        };
        let (ordered, publisher) = order_confs(dir, &confs);
        found.push(DiscoveredInstall {
            game_id: game.id.clone(),
            publisher,
            root: dir.to_path_buf(),
            saves,
            confs: ordered,
            emulator: bundled_emulator(dir),
        });
    }
    found
}

/// The `.conf` files directly in `dir`, each with whether it holds an
/// `[autoexec]` mount.
fn conf_files(dir: &Path) -> Vec<(String, bool)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut confs: Vec<(String, bool)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_str()?.to_string();
            if !name.to_ascii_lowercase().ends_with(".conf") || !path.is_file() {
                return None;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            Some((name, has_autoexec_mount(&text)))
        })
        .collect();
    confs.sort();
    confs
}

/// Whether a conf's `[autoexec]` section contains a `mount` line.
fn has_autoexec_mount(text: &str) -> bool {
    let mut in_autoexec = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_autoexec = line.eq_ignore_ascii_case("[autoexec]");
            continue;
        }
        if in_autoexec && line.to_ascii_lowercase().starts_with("mount") {
            return true;
        }
    }
    false
}

/// Puts the confs in launch order.
///
/// The publisher's launch script is the authority when one is present. When
/// none is, settings files come first and the one holding `[autoexec]` comes
/// last, which is the rule both publishers follow.
fn order_confs(dir: &Path, confs: &[(String, bool)]) -> (Vec<String>, Option<Publisher>) {
    for (script, publisher) in [
        ("start.sh", Publisher::Gog),
        ("run-game.bat", Publisher::Steam),
    ] {
        let Ok(text) = std::fs::read_to_string(dir.join(script)) else {
            continue;
        };
        let scripted = confs_named_in(&text);
        // The script decides the order; the directory decides existence. A
        // conf the script names but the disk lacks is dropped, and a conf the
        // script skips is not launched.
        let ordered: Vec<String> = scripted
            .into_iter()
            .filter(|name| confs.iter().any(|(c, _)| c == name))
            .collect();
        if !ordered.is_empty() {
            return (ordered, Some(publisher));
        }
    }

    let mut ordered: Vec<String> = confs
        .iter()
        .filter(|(_, autoexec)| !autoexec)
        .map(|(name, _)| name.clone())
        .collect();
    ordered.extend(
        confs
            .iter()
            .filter(|(_, autoexec)| *autoexec)
            .map(|(name, _)| name.clone()),
    );
    (ordered, None)
}

/// The conf file names a launch script mentions, in order.
///
/// Steam's `run-game.bat` passes each as `-conf name.conf`. GOG's `start.sh`
/// passes them as quoted arguments to its `run_dosbox` helper, which adds the
/// `-conf` flags itself. Collecting every token that names a `.conf` file
/// covers both without knowing either helper.
fn confs_named_in(script: &str) -> Vec<String> {
    script
        .split_whitespace()
        .filter_map(|token| {
            let name = token
                .trim_matches('"')
                .trim_start_matches(".\\")
                .trim_end_matches('\r');
            if !name.to_ascii_lowercase().ends_with(".conf") {
                return None;
            }
            let leaf = name.rsplit(['/', '\\']).next().unwrap_or(name);
            Some(leaf.to_string())
        })
        .collect()
}

/// The folder holding `CHRDAT*.SAV` files inside a folder named like the
/// game's save folder, relative to the install root.
///
/// GOG keeps the saves in the game folder itself (`data/POOLRAD`). Steam
/// keeps them one level inside it (`GAME/POOLRAD/SAVE`), so a direct child
/// holding `CHRDAT` files counts too. Children are tried in sorted order, so
/// the pick is stable.
fn find_save_folder(root: &Path, folder_name: &str) -> Option<PathBuf> {
    let mut dirs = Vec::new();
    collect_dirs(root, 0, &mut dirs);
    for dir in dirs {
        let name = dir.file_name()?.to_str()?;
        if !name.eq_ignore_ascii_case(folder_name) {
            continue;
        }
        if has_chrdat_files(&dir) {
            return dir.strip_prefix(root).ok().map(Path::to_path_buf);
        }
        let mut children: Vec<PathBuf> = std::fs::read_dir(&dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        children.sort();
        for child in children {
            if has_chrdat_files(&child) {
                return child.strip_prefix(root).ok().map(Path::to_path_buf);
            }
        }
    }
    None
}

fn has_chrdat_files(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let name = e.file_name().to_string_lossy().to_ascii_uppercase();
        name.starts_with("CHRDAT") && name.ends_with(".SAV")
    })
}

/// An executable named `dosbox` shipped inside the install.
///
/// GOG ships both `dosbox/` and `dosbox-staging/`. Staging is preferred: it is
/// the build GOG launches on current installs, and the one with working sound
/// and shaders on Linux.
fn bundled_emulator(root: &Path) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let mut dirs = Vec::new();
    collect_dirs(root, 0, &mut dirs);
    let mut candidates: Vec<PathBuf> = dirs
        .iter()
        .filter_map(|dir| {
            let bin = dir.join("dosbox");
            let meta = std::fs::metadata(&bin).ok()?;
            (meta.is_file() && meta.permissions().mode() & 0o111 != 0).then_some(bin)
        })
        .collect();
    candidates.sort_by_key(|p| {
        // Sort staging first, then shortest path, so the pick is stable.
        (!p.to_string_lossy().contains("staging"), p.clone())
    });
    candidates.into_iter().next()
}

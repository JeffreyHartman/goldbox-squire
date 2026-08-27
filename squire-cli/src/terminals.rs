//! Which terminals Squire can open a window in, and how to ask each one.
//!
//! What the table holds and how to add a terminal is written for the user, at
//! the top of `terminals.toml`. That file is the explanation; this one is the
//! reading of it, and the two must not both grow prose or they will disagree.
//!
//! The one thing worth saying here: the defaults are compiled in and the
//! user's file is merged over them by name. That is the same principle as
//! another game being a table rather than code, with one difference. A record
//! table is Squire's. A terminal entry can be the user's, which is why a bad
//! one complains and carries on rather than stopping the program.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The compiled-in table. Parsed, not read from disk.
const BUILT_IN: &str = include_str!("../terminals.toml");

/// A kind of window Squire opens: the HUD today, a map or a journal later.
///
/// Each kind has one owned window name, which is what a compositor rule
/// matches on. The user writes one rule per window, by hand, so the names are
/// owned here rather than passed in as strings: a caller free to pass any name
/// is a caller free to break every rule already written. The parameter is a
/// kind rather than a name for exactly that reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    Hud,
}

impl ViewKind {
    /// Every kind there is. A new view adds one arm here and one name below.
    pub const ALL: [ViewKind; 1] = [ViewKind::Hud];

    /// The name this view's window reports to the desktop.
    ///
    /// Changing one of these breaks every compositor rule already written
    /// against it, silently, which is why they are pinned by a test.
    pub fn app_id(self) -> &'static str {
        match self {
            ViewKind::Hud => "goldbox-squire-hud",
        }
    }

    /// What the user types to ask for this view.
    pub fn as_str(self) -> &'static str {
        match self {
            ViewKind::Hud => "hud",
        }
    }

    pub fn parse(word: &str) -> Option<ViewKind> {
        ViewKind::ALL.into_iter().find(|k| k.as_str() == word)
    }
}

/// The placeholders an entry may use. Anything else is a typo, and a typo
/// that reached the command line would be a window nobody can find.
const PLACEHOLDERS: [&str; 3] = ["{id}", "{cols}", "{rows}"];

/// One terminal, and how to ask it for a named window of a given size.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Terminal {
    /// The program, as it is spelled on PATH.
    pub name: String,
    /// The arguments that name the window. Uses `{id}`.
    #[serde(default)]
    pub app_id: Vec<String>,
    /// The arguments that ask for a size in cells. Uses `{cols}` and `{rows}`.
    #[serde(default)]
    pub size: Vec<String>,
    /// What goes between the options and the command. Some terminals need a
    /// flag here and some need nothing, so leaving it out means nothing.
    #[serde(default)]
    pub exec: Vec<String>,
}

impl Terminal {
    /// The whole command line: the terminal, its options, then the command.
    ///
    /// The command is always last, because at least one terminal refuses to
    /// read anything after it.
    pub fn command_line(
        &self,
        view: ViewKind,
        cols: u16,
        rows: u16,
        command: &[String],
    ) -> Vec<String> {
        let fill = |arg: &String| {
            arg.replace("{id}", view.app_id())
                .replace("{cols}", &cols.to_string())
                .replace("{rows}", &rows.to_string())
        };
        let mut argv = vec![self.name.clone()];
        argv.extend(self.app_id.iter().map(&fill));
        argv.extend(self.size.iter().map(&fill));
        argv.extend(self.exec.iter().map(&fill));
        argv.extend(command.iter().cloned());
        argv
    }

    /// Every `{...}` in the entry that is not a placeholder Squire knows.
    fn unknown_placeholders(&self) -> Vec<String> {
        let mut found = Vec::new();
        for arg in self.app_id.iter().chain(&self.size).chain(&self.exec) {
            let mut rest = arg.as_str();
            while let Some(open) = rest.find('{') {
                let Some(close) = rest[open..].find('}') else {
                    break;
                };
                let word = &rest[open..open + close + 1];
                if !PLACEHOLDERS.contains(&word) && !found.iter().any(|f| f == word) {
                    found.push(word.to_string());
                }
                rest = &rest[open + close + 1..];
            }
        }
        found
    }
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    terminal: Vec<Terminal>,
}

/// The user's file, read one entry at a time.
///
/// Each block stays raw until it is read on its own, so that one bad block
/// names itself and the good blocks around it still take effect. Reading the
/// file into `Vec<Terminal>` in one go would throw away a whole file over one
/// misspelled field.
#[derive(Deserialize)]
struct RawFile {
    #[serde(default)]
    terminal: Vec<toml::Value>,
}

/// The compiled-in terminals.
///
/// This cannot fail on a correct build: the same file is parsed by the test
/// suite, so a typo fails the tests rather than reaching a user.
pub fn built_in() -> Vec<Terminal> {
    parse(BUILT_IN).expect("the built-in terminal table is valid")
}

fn parse(text: &str) -> Result<Vec<Terminal>, String> {
    toml::from_str::<File>(text)
        .map(|f| f.terminal)
        .map_err(|e| e.to_string())
}

/// The user's file, merged over `defaults` by terminal name.
///
/// Returns what to use and what to complain about. A bad file, or a bad entry
/// in a good file, is never fatal: Squire keeps running with the entries it
/// could read, because a HUD that will not start over a stale config file is
/// worse than a HUD at the wrong size.
///
/// `whence` is the file's name, used in the complaints so that the user knows
/// which file to edit.
pub fn merge(
    defaults: Vec<Terminal>,
    user_text: &str,
    whence: &str,
) -> (Vec<Terminal>, Vec<String>) {
    let mut list = defaults;
    let mut problems = Vec::new();

    let blocks = match toml::from_str::<RawFile>(user_text) {
        Ok(file) => file.terminal,
        Err(e) => {
            problems.push(format!("{whence} could not be read and was ignored: {e}"));
            return (list, problems);
        }
    };

    for (i, block_value) in blocks.into_iter().enumerate() {
        // One-based, because the user counts blocks in a file from one.
        let block = i + 1;
        let entry: Terminal = match block_value.try_into() {
            Ok(entry) => entry,
            Err(e) => {
                problems.push(format!("{whence}: terminal {block} was ignored: {e}"));
                continue;
            }
        };
        if entry.name.trim().is_empty() {
            problems.push(format!(
                "{whence}: terminal {block} has no `name` and was ignored"
            ));
            continue;
        }
        let unknown = entry.unknown_placeholders();
        if !unknown.is_empty() {
            problems.push(format!(
                "{whence}: terminal {block} (`{}`) uses {}, which Squire does not \
                 know. The placeholders are {}. The entry was ignored.",
                entry.name,
                unknown.join(", "),
                PLACEHOLDERS.join(", ")
            ));
            continue;
        }
        match list.iter().position(|t| t.name == entry.name) {
            Some(existing) => list[existing] = entry,
            None => list.push(entry),
        }
    }

    (list, problems)
}

/// The entry for a terminal, found by program name.
///
/// `program` may be a path, because that is what a `TERMINAL` variable or a
/// `--terminal` argument often holds. Only the file name is compared.
pub fn find<'a>(list: &'a [Terminal], program: &str) -> Option<&'a Terminal> {
    let name = Path::new(program).file_name()?.to_str()?;
    list.iter().find(|t| t.name == name)
}

/// The user's terminal file, beside the config file.
pub fn path() -> Option<PathBuf> {
    crate::config::Config::path().map(|p| p.with_file_name("terminals.toml"))
}

/// Every terminal Squire knows, with the user's file merged over the defaults.
///
/// A missing user file is the normal case and says nothing.
pub fn load() -> (Vec<Terminal>, Vec<String>) {
    let Some(p) = path() else {
        return (built_in(), Vec::new());
    };
    let Ok(text) = std::fs::read_to_string(&p) else {
        return (built_in(), Vec::new());
    };
    merge(built_in(), &text, &p.display().to_string())
}

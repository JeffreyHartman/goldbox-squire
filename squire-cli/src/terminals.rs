//! Which terminals Squire can open a window in, and how to ask each one.
//!
//! Squire needs two things from a terminal: a name it can give the window, so
//! that a compositor rule can find it, and a size in character cells rather
//! than pixels. Every terminal spells both differently, and there is no
//! standard, so this is data.
//!
//! The defaults are compiled in, and a user file is merged over them by
//! terminal name. This is the same principle as another game being a table
//! rather than code, with one difference worth stating: a game table is ours,
//! and a terminal entry can be theirs. Somebody's favourite terminal in five
//! years does not exist yet, and adding it must be a file they write, not a
//! pull request they send.
//!
//! A terminal that is in no table is not an error. It is launched anyway, and
//! Squire says once that the size could not be set.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The compiled-in table. Parsed, not read from disk.
const BUILT_IN: &str = include_str!("../terminals.toml");

/// The placeholders an entry may use. Anything else is a typo, and a typo
/// that reached the command line would be a window nobody can find.
const PLACEHOLDERS: [&str; 3] = ["{id}", "{cols}", "{rows}"];

/// One terminal, and how to ask it for a named window of a given size.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Terminal {
    /// The program, as it is spelled on PATH.
    pub name: String,
    /// The arguments that name the window. Uses `{id}`.
    pub app_id: Vec<String>,
    /// The arguments that ask for a size in cells. Uses `{cols}` and `{rows}`.
    pub size: Vec<String>,
    /// What goes between the options and the command. Some terminals need a
    /// flag here and some need nothing.
    pub exec: Vec<String>,
}

impl Terminal {
    /// The whole command line: the terminal, its options, then the command.
    ///
    /// The command is always last, because at least one terminal refuses to
    /// read anything after it.
    pub fn command_line(&self, id: &str, cols: u16, rows: u16, command: &[String]) -> Vec<String> {
        let fill = |arg: &String| {
            arg.replace("{id}", id)
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

    let entries = match parse(user_text) {
        Ok(entries) => entries,
        Err(e) => {
            problems.push(format!("{whence} could not be read and was ignored: {e}"));
            return (list, problems);
        }
    };

    for (i, entry) in entries.into_iter().enumerate() {
        // One-based, because the user counts blocks in a file from one.
        let at = i + 1;
        if entry.name.trim().is_empty() {
            problems.push(format!(
                "{whence}: terminal {at} has no `name` and was ignored"
            ));
            continue;
        }
        let unknown = entry.unknown_placeholders();
        if !unknown.is_empty() {
            problems.push(format!(
                "{whence}: terminal {at} (`{}`) uses {}, which Squire does not \
                 know. The placeholders are {}. The entry was ignored.",
                entry.name,
                unknown.join(", "),
                PLACEHOLDERS.join(", ")
            ));
            continue;
        }
        match list.iter().position(|t| t.name == entry.name) {
            Some(at) => list[at] = entry,
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

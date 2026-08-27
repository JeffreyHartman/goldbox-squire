//! Opening a view in a window of its own.
//!
//! Which terminal, and what to say to it. The table of terminals is
//! [`crate::terminals`]; this is the choosing and the command line built from
//! it, kept apart so that both can be tested without opening a window.

use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::layout::Size;
use crate::terminals::{self, Terminal, ViewKind};

/// Which terminal to open the window in.
///
/// In order: what the user asked for, the `TERMINAL` variable, then the first
/// terminal Squire knows that is installed. A terminal the user named is used
/// whether Squire knows it or not, because the user naming one is the whole
/// answer to "which terminal", and second-guessing it would be the hidden
/// trap that arguing about the size is not.
pub fn choose(
    list: &[Terminal],
    asked: Option<&str>,
    from_env: Option<&str>,
    on_path: &dyn Fn(&str) -> bool,
) -> Option<String> {
    // Each is tried in turn and an empty one is no answer at all, so
    // `--terminal ""` falls through to TERMINAL rather than silently
    // throwing it away.
    let named = [asked, from_env]
        .into_iter()
        .flatten()
        .find(|name| !name.trim().is_empty());
    if let Some(named) = named {
        return Some(named.to_string());
    }
    list.iter()
        .map(|t| t.name.clone())
        .find(|name| on_path(name))
}

/// The command line for the window, and what to say about it if anything.
///
/// A terminal Squire does not know is still opened. It costs the size and the
/// window name, which is worth one sentence, and nothing else. There is no
/// second route through this function for the unknown case, because two
/// routes through one feature is how one of them rots.
pub fn plan(
    list: &[Terminal],
    program: &str,
    kind: ViewKind,
    size: Size,
    command: &[String],
) -> (Vec<String>, Option<String>) {
    match terminals::find(list, program) {
        Some(terminal) => (
            terminal.command_line(kind, size.cols, size.rows, command),
            None,
        ),
        None => {
            let mut argv = vec![program.to_string()];
            argv.extend(command.iter().cloned());
            (
                argv,
                Some(format!(
                    "gbs does not know {program}, so it could not set the window's \
                     size or name. The window still opens. Adding an entry for it \
                     to terminals.toml in gbs's config folder fixes both."
                )),
            )
        }
    }
}

/// The command a view is started with: this same program, told where to draw.
pub fn view_command(gbs: &Path, kind: ViewKind, socket: &Path) -> Vec<String> {
    vec![
        gbs.display().to_string(),
        "--view".into(),
        kind.as_str().into(),
        "--socket".into(),
        socket.display().to_string(),
    ]
}

/// Whether `name` is an executable on PATH.
pub fn on_path(name: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        std::fs::metadata(dir.join(name))
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    })
}

/// Opens the window.
///
/// The terminal's own output is discarded. A terminal that has something to
/// say says it in the window it opened, and letting it write here would put
/// it on top of the log.
pub fn open(argv: &[String]) -> Result<Child, String> {
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| "no terminal to open".to_string())?;
    Command::new(program)
        .args(rest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("opening a window with {program}: {e}"))
}

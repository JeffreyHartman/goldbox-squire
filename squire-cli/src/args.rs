//! The command line arguments.
//!
//! Parsed by hand. The tool has nine flags, and a hand-written parser keeps the
//! dependency list short enough to read in one sitting.

use std::fmt;

/// What the tool was asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Print the party once, then exit.
    #[default]
    Once,
    /// Redraw until the user stops it.
    Watch,
    /// Print the usage text.
    Help,
}

/// Everything the command line said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    pub game_dir: Option<String>,
    pub dosbox: Option<String>,
    pub conf: Option<String>,
    /// Read an emulator this tool did not start. Needs a relaxed
    /// `kernel.yama.ptrace_scope`, so it is not the normal path.
    pub pid: Option<i32>,
    pub json: bool,
    pub mode: Mode,
    /// Milliseconds between redraws in watch mode.
    pub interval_ms: u64,
}

/// A bad command line.
#[derive(Debug)]
pub struct ArgError(String);

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ArgError {}

pub const USAGE: &str = "\
gbs — Goldbox Squire. Shows the live party of an SSI Gold Box game.

USAGE:
    gbs [OPTIONS]

OPTIONS:
    --game-dir <DIR>   The folder holding the game and its CHRDATA*.SAV files.
                       Stored in the config file, so it is needed once.
    --dosbox <CMD>     The emulator to start. Default: dosbox
    --conf <FILE>      A configuration file to pass to the emulator.
    --pid <PID>        Read an emulator this tool did not start. This works
                       only where the system already permits it, and gbs will
                       say so if it does not. Letting gbs start the game is
                       the supported path and needs no system change.
    --watch            Redraw until stopped, rather than printing once.
    --interval <MS>    Milliseconds between redraws. Default: 500
    --json             Print JSON rather than a table.
    -h, --help         Print this text.

Goldbox Squire starts the emulator itself. That is what makes reading its
memory permitted without changing any system setting.
";

impl Args {
    /// Parses the arguments, which must not include the program name.
    pub fn parse(argv: impl IntoIterator<Item = String>) -> Result<Args, ArgError> {
        let mut args = Args {
            interval_ms: 500,
            ..Default::default()
        };
        let mut it = argv.into_iter();

        while let Some(arg) = it.next() {
            let mut value = |flag: &str| -> Result<String, ArgError> {
                it.next()
                    .ok_or_else(|| ArgError(format!("{flag} needs a value")))
            };
            match arg.as_str() {
                "--game-dir" => args.game_dir = Some(value("--game-dir")?),
                "--dosbox" => args.dosbox = Some(value("--dosbox")?),
                "--conf" => args.conf = Some(value("--conf")?),
                "--pid" => {
                    let raw = value("--pid")?;
                    args.pid = Some(
                        raw.parse()
                            .map_err(|_| ArgError(format!("--pid needs a number, got `{raw}`")))?,
                    );
                }
                "--interval" => {
                    let raw = value("--interval")?;
                    args.interval_ms = raw
                        .parse()
                        .map_err(|_| ArgError(format!("--interval needs a number, got `{raw}`")))?;
                }
                "--json" => args.json = true,
                "--watch" => args.mode = Mode::Watch,
                "-h" | "--help" => args.mode = Mode::Help,
                other => {
                    return Err(ArgError(format!(
                        "unknown option `{other}`. Run `gbs --help` for the list."
                    )))
                }
            }
        }

        Ok(args)
    }
}

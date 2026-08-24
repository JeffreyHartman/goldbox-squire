//! The command line arguments.
//!
//! Parsed by hand. The tool has ten flags, and a hand-written parser keeps the
//! dependency list short enough to read in one sitting.
//!
//! There is no `--watch`: watching is what the tool does. An argument that is
//! required to make the program work is not an argument.

use std::fmt;

/// Everything the command line said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Args {
    /// Answers the wizard's first question in advance: which game.
    pub game: Option<String>,
    /// Answers the second: which save slot, a letter A through J.
    pub slot: Option<char>,
    pub game_dir: Option<String>,
    pub dosbox: Option<String>,
    pub conf: Option<String>,
    /// Read an emulator this tool did not start. Needs a relaxed
    /// `kernel.yama.ptrace_scope`, so it is not the normal path.
    pub pid: Option<i32>,
    pub json: bool,
    pub help: bool,
    /// Milliseconds between redraws once a party was found.
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

A bare `gbs` picks the install and the save slot, starts the game, waits for
the party, and redraws it until the emulator exits or you stop it. Each option
below answers one of those questions in advance.

OPTIONS:
    --game <ID>        Which game to run, by its id (pool-of-radiance).
    --slot <A-J>       Which save slot to read. Asked every run otherwise,
                       because a slot describes one sitting.
    --interval <MS>    Milliseconds between redraws. Default: 500
    --json             Print JSON rather than a table.
    --pid <PID>        Read an emulator this tool did not start. This works
                       only where the system already permits it, and gbs will
                       say so if it does not. Letting gbs start the game is
                       the supported path and needs no system change.
    -h, --help         Print this text.

MANUAL SETUP, when discovery does not find your install:
    --game-dir <DIR>   The folder holding the game's CHRDAT*.SAV files.
    --conf <FILE>      The DOSBox configuration file that starts the game.
    --dosbox <CMD>     The emulator to start. Default: the first of dosbox,
                       dosbox-staging, dosbox-x found on PATH.
    The three are remembered as a manual install, so they are given once.

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
                "--game" => args.game = Some(value("--game")?),
                "--slot" => {
                    let raw = value("--slot")?;
                    args.slot = Some(parse_slot(&raw)?);
                }
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
                "-h" | "--help" => args.help = true,
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

/// A save slot is one letter, A through J.
fn parse_slot(raw: &str) -> Result<char, ArgError> {
    let mut chars = raw.chars();
    let (Some(letter), None) = (chars.next(), chars.next()) else {
        return Err(ArgError(format!(
            "--slot needs one letter A through J, got `{raw}`"
        )));
    };
    let letter = letter.to_ascii_uppercase();
    if !letter.is_ascii_uppercase() || letter > 'J' {
        return Err(ArgError(format!(
            "--slot needs one letter A through J, got `{raw}`"
        )));
    }
    Ok(letter)
}

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
    /// Answers Unlimited Adventures' extra question: which design
    /// (adventure module), by name, case-insensitively.
    pub design: Option<String>,
    /// Points the game at this game folder, and remembers it (ADR 0004).
    pub game_dir: Option<String>,
    /// The emulator for this one run. The config's `dosbox` field is the
    /// permanent version.
    pub dosbox: Option<String>,
    /// Read an emulator this tool did not start. Needs a relaxed
    /// `kernel.yama.ptrace_scope`, so it is not the normal path.
    pub pid: Option<i32>,
    pub json: bool,
    /// Print the table instead of opening the HUD. For pipes, scripts and
    /// anything reading `gbs` as text.
    pub plain: bool,
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

A bare `gbs` asks which game, where it is (the first time only; the answer is
remembered per game), and which save slot. Unlimited Adventures keeps saves
per adventure module, so it gets one more question: which adventure. Then gbs
starts the game, waits for the party, and shows it on a screen you can glance
at until the emulator exits or you press q. Each option below answers one of those questions in advance.
In the menus, 0 goes back. A fresh install with no saved game can still be
started: save inside the game, then press Enter in gbs to pick the save.

OPTIONS:
    --game <ID>        Which game to run, by its id (pool-of-radiance).
    --game-dir <DIR>   Use this game folder for the game, and remember it.
                       The wizard's directory question also takes a typed
                       path, so this flag is never required.
    --slot <A-J>       Which save slot to read. Asked every run otherwise,
                       because a slot describes one sitting.
    --design <NAME>    Which adventure, for Unlimited Adventures: the design's
                       name, as its folder is called (BASILISK for
                       BASILISK.DSN, any case). Asked every run otherwise.
    --dosbox <CMD>     The emulator for this run. Default: the first of
                       dosbox, dosbox-staging, dosbox-x found on PATH. A
                       `dosbox` line in the config file makes it permanent.
    --interval <MS>    Milliseconds between redraws. Default: 500
    --plain            Print a table into this terminal and keep reprinting
                       it, rather than opening the HUD. For pipes, scripts
                       and anything reading gbs as text. Implied by --json.
    --json             Print JSON rather than a table.
    --pid <PID>        Read an emulator this tool did not start. This works
                       only where the system already permits it, and gbs will
                       say so if it does not. Letting gbs start the game is
                       the supported path and needs no system change.
    -h, --help         Print this text.

In the HUD: q quits, up and down move the highlight, a shows the ability
scores, c changes how many cards sit across, and Enter picks a different save
slot. The size you leave the window at is remembered in gbs's config file,
under [hud], and used next time.

Emulator settings live in a per-game file gbs creates in its config folder
and never touches again; the first launch names it.

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
                "--design" => args.design = Some(value("--design")?),
                "--dosbox" => args.dosbox = Some(value("--dosbox")?),
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
                "--plain" => args.plain = true,
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

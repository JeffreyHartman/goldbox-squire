//! `gbs` — the command line front end.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use squire_cli::args::{Args, USAGE};
use squire_cli::attach;
use squire_cli::conf;
use squire_cli::config::{Config, WindowSize};
use squire_cli::hud;
use squire_cli::keys;
use squire_cli::layout::{Caption, Size};
use squire_cli::manual;
use squire_cli::output;
use squire_cli::view;
use squire_cli::watch::{self, Watch};
use squire_cli::wizard;
use squire_core::launch::Emulator;
use squire_core::mem::ProcessReader;
use squire_core::session::Session;
use squire_core::{games, saves};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("gbs: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(std::env::args().skip(1)).map_err(|e| e.to_string())?;

    if args.help {
        print!("{USAGE}");
        return Ok(());
    }

    let mut config = Config::load();

    // A view is a window on a run that is already going, so it starts no game
    // and asks no questions. This is how gbs opens its own windows, and
    // running it by hand is how a second window is opened on a sitting.
    if let Some(kind) = args.view {
        let socket = args
            .socket
            .clone()
            .expect("the parser pairs --view with --socket");
        let remembered = config.hud.map(|h| Size {
            cols: h.columns,
            rows: h.rows,
        });
        let size = view::run(kind, Path::new(&socket), remembered)?;
        remember_size(&mut config, size);
        return Ok(());
    }

    // Attaching to an emulator this tool did not start is the unusual path.
    // It needs a relaxed kernel.yama.ptrace_scope, so it is never the
    // default, and it is the automation path: no wizard, no launch, no
    // discovery, and nothing written to the config.
    if let Some(pid) = args.pid {
        let resolved = attach::resolve(&args, &config)?;
        let game = games::find(&resolved.game_id).expect("resolve validated the game id");
        let mut session = Session::new(
            ProcessReader::new(pid),
            game.table.clone(),
            resolved.names.clone(),
        );
        let watching = Watching {
            game,
            slot: Some(resolved.slot),
            names: resolved.names,
            save_dir: None,
        };
        return run_watch(&mut session, &args, &watching, None, &mut config);
    }

    // Find the user's installs, once. A normal run reads the cached results;
    // the scan reruns only when there are none, a stored root vanished, or
    // this build knows games the last scan did not look for. Finding nothing
    // is not fatal: the wizard's directory question takes a typed path.
    let game_ids: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
    if config.installs.is_empty() || config.needs_rediscovery() || config.known_games != game_ids {
        let mut roots = squire_core::discover::default_roots();
        roots.extend(config.extra_roots.iter().map(PathBuf::from));
        let remembered = config.installs.clone();
        let absorbed = config.absorb(squire_core::discover::discover(&roots));
        // A rescan is the authority on discovered installs, but losing one
        // silently would read as gbs forgetting. Say what was dropped and
        // how to get it back. A manual entry collapsed into a rediscovered
        // install of the same folder was not lost, only renamed, so it does
        // not count.
        for (key, install) in &remembered {
            if config.installs.contains_key(key) {
                continue;
            }
            let dir = std::fs::canonicalize(install.save_dir()).unwrap_or(install.save_dir());
            let still_covered = config.installs.values().any(|other| {
                other.game == install.game
                    && std::fs::canonicalize(other.save_dir()).unwrap_or(other.save_dir()) == dir
            });
            if !still_covered {
                eprintln!(
                    "gbs: {} at {} no longer looks like an install and was \
                     dropped from the list. If it is one, point at it again \
                     with --game-dir.",
                    install.game, install.root
                );
            }
        }
        let registry_grew = config.known_games != game_ids;
        config.known_games = game_ids;
        if absorbed || registry_grew {
            if let Err(e) = config.save() {
                eprintln!("gbs: warning: could not save the settings: {e}");
            }
        }
    }

    let before = config.clone();
    let (key, sitting) = wizard::choose(
        &mut std::io::stdin().lock(),
        &mut std::io::stderr(),
        &mut config,
        args.game.as_deref(),
        args.game_dir.as_deref(),
        args.design.as_deref(),
        args.slot,
    )?;
    if config != before {
        if let Err(e) = config.save() {
            eprintln!("gbs: warning: could not save the settings: {e}");
        }
    }
    let install = config.installs[&key].clone();
    let game = games::find(&install.game)
        .ok_or_else(|| format!("unknown game `{}` in the config", install.game))?;
    // No sitting means a fresh install: launch with no party to look for,
    // and the user picks the save mid-watch once it exists.
    let (slot, names) = match &sitting {
        Some(sitting) => (
            Some(sitting.slot),
            saves::slot_party_names(&game, &sitting.save_dir, sitting.slot)
                .map_err(|e| e.to_string())?,
        ),
        None => (None, Vec::new()),
    };
    let table = game.table.clone();

    // A hand-named folder can disagree with where the game's own DOS config
    // points; refuse that before launching. Discovery cannot mis-name one.
    manual::folder_name_check(&install, &game)?;

    // The normal path. Starting the emulator is what makes the read permitted.
    // Every install launches the same way (ADR 0004): the per-game settings
    // conf the user edits, plus an autoexec computed from where the game is.
    let command = squire_cli::emulator::command(
        args.dosbox.as_deref(),
        config.dosbox.as_deref(),
        squire_cli::emulator::find(),
    )?;
    let mut emulator = Emulator::new(&command);
    let (settings, created) = conf::ensure(&own_config_dir()?, &game)?;
    if created {
        eprintln!(
            "gbs: created {}. Emulator settings live there and are yours to edit.",
            settings.display()
        );
    }
    emulator = emulator.conf(&settings);
    for dos_command in conf::autoexec(&install, &game)? {
        emulator = emulator.command(dos_command);
    }
    let log = log_path();
    emulator = emulator.log_to(&log);

    let mut running = emulator.start().map_err(|e| e.to_string())?;
    eprintln!(
        "gbs: started {command} as process {}. Its messages go to {}",
        running.pid(),
        log.display()
    );

    let mut session = Session::new(running.reader(), table, names.clone());
    // The repick folder is the install's own save folder: for a designs game
    // that is the game folder, and repicking asks the design again, so a
    // fresh install's first save is reachable mid-watch.
    let watching = Watching {
        game,
        slot,
        names,
        save_dir: Some(install.save_dir()),
    };
    run_watch(
        &mut session,
        &args,
        &watching,
        Some(&mut running),
        &mut config,
    )
}

/// Wires the watch loop to a front end and a keyboard.
///
/// The HUD is what a person gets, and `--plain` is the escape. `args.rs`
/// already argues that an argument required to make the program work is not
/// an argument, which is why there is no `--watch`; the same reasoning makes
/// the HUD the default rather than a `--tui` opt-in.
fn run_watch<R: squire_core::mem::Reader>(
    session: &mut Session<R>,
    args: &Args,
    watching: &Watching,
    running: Option<&mut squire_core::launch::Launched>,
    config: &mut Config,
) -> Result<(), String> {
    let Watching {
        game,
        slot,
        names,
        save_dir,
    } = watching;
    let (slot, names, save_dir) = (*slot, names.clone(), save_dir.as_deref());
    let timing = Watch {
        interval: Duration::from_millis(args.interval_ms),
        ..Watch::default()
    };
    let running = running.map(|r| r as &mut dyn watch::Alive);

    // JSON is for a program, and a program is never watching a screen. A
    // stream that is not a terminal cannot hold a HUD at all, and saying so
    // beats failing to take over something that is not there.
    let plain = args.plain || args.json || !is_terminal();
    if plain && !(args.plain || args.json) {
        eprintln!("gbs: standard output is not a terminal, so the table is printed instead.");
    }

    if plain {
        let mut screen = output::Plain::stdio(args.json);
        let mut keys = keys::Stdin::new(game, save_dir);
        return watch::watch(
            session,
            &timing,
            &mut screen,
            &mut keys,
            running,
            slot,
            names,
        );
    }

    let caption = Caption {
        game: game.name.clone(),
        slot,
        panel: "party".to_string(),
        note: None,
    };
    let remembered = config.hud.map(|h| Size {
        cols: h.columns,
        rows: h.rows,
    });
    let interface = hud::Hud::start(caption, remembered)?;
    let mut screen = interface.screen();
    let mut keys = interface.keys(game, save_dir);
    let outcome = watch::watch(
        session,
        &timing,
        &mut screen,
        &mut keys,
        running,
        slot,
        names,
    );

    // Recorded on the way out, whether the run ended well or badly: the user
    // resized the window either way, and losing that over an error would mean
    // resizing it again next launch.
    // Read before the terminal goes back, written after: a warning printed
    // onto the alternate screen is a warning nobody can read.
    let size = interface.size();
    drop(interface);
    remember_size(config, size);
    outcome
}

/// What one watch is pointed at.
///
/// These four travel together everywhere below the wizard, and the `--pid`
/// path fills the same four from a different source. One type rather than one
/// long parameter list.
struct Watching {
    game: games::Game,
    /// The save slot, when one has been picked. A fresh install has none until
    /// the user saves in game and repicks mid-watch.
    slot: Option<char>,
    /// The character names to look for, taken from the save files.
    names: Vec<String>,
    /// The install's own save folder, which is where a repick starts looking.
    /// `None` on the `--pid` path, which never started a game.
    save_dir: Option<PathBuf>,
}

/// Writes the size the HUD was left at into the config.
///
/// A size of zero is what a terminal reports when it does not know its own,
/// and it is not worth writing down.
fn remember_size(config: &mut Config, size: Size) {
    let hud = WindowSize {
        columns: size.cols,
        rows: size.rows,
    };
    if !hud.looks_like_a_window() || config.hud == Some(hud) {
        return;
    }
    config.hud = Some(hud);
    if let Err(e) = config.save() {
        eprintln!("gbs: warning: could not remember the window size: {e}");
    }
}

/// Whether standard output is a terminal, asked of the kernel rather than of
/// an environment variable, because a variable is a guess.
fn is_terminal() -> bool {
    use crossterm::tty::IsTty;
    std::io::stdout().is_tty()
}

/// The folder gbs's own files live in: the config file's folder.
///
/// The settings conf is promised to the user as theirs to keep, so a machine
/// with no config folder is an error, never a silent file in /tmp.
fn own_config_dir() -> Result<PathBuf, String> {
    Config::path()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .ok_or_else(|| "no config folder found. Set HOME or XDG_CONFIG_HOME.".into())
}

/// Where the emulator's output goes, next to the config file.
fn log_path() -> PathBuf {
    Config::path()
        .and_then(|p| p.parent().map(|d| d.join("emulator.log")))
        .unwrap_or_else(|| std::env::temp_dir().join("gbs-emulator.log"))
}

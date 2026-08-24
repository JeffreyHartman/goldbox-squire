//! `gbs` — the command line front end.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use squire_cli::args::{Args, USAGE};
use squire_cli::attach;
use squire_cli::wizard;
use squire_cli::conf;
use squire_cli::config::Config;
use squire_cli::manual;
use squire_cli::output;
use squire_core::launch::{Emulator, Launched};
use squire_core::mem::ProcessReader;
use squire_core::session::{PartyState, Session};
use squire_core::{games, saves, Error};

/// How often to poll while no party was found yet. Each failed poll is a full
/// memory sweep through DOS boot, title screen and load menu, so this is far
/// slower than the redraw interval.
const WAITING_POLL: Duration = Duration::from_secs(2);

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
        return watch(
            &mut session,
            &args,
            &game,
            None,
            resolved.slot,
            resolved.names,
            None,
        );
    }

    // Find the user's installs, once. A normal run reads the cached results;
    // the scan reruns only when there are none, a stored root vanished, or
    // this build knows games the last scan did not look for. Finding nothing
    // is not fatal: the wizard's directory question takes a typed path.
    let game_ids: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
    if config.installs.is_empty() || config.needs_rediscovery() || config.known_games != game_ids
    {
        let mut roots = squire_core::discover::default_roots();
        roots.extend(config.extra_roots.iter().map(PathBuf::from));
        let absorbed = config.absorb(squire_core::discover::discover(&roots));
        let registry_grew = config.known_games != game_ids;
        config.known_games = game_ids;
        if absorbed || registry_grew {
            if let Err(e) = config.save() {
                eprintln!("gbs: warning: could not save the settings: {e}");
            }
        }
    }

    let before = config.clone();
    let (key, save_dir, slot) = wizard::choose(
        &mut std::io::stdin().lock(),
        &mut std::io::stderr(),
        &mut config,
        args.game.as_deref(),
        args.game_dir.as_deref(),
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
    let names = saves::slot_party_names(&game, &save_dir, slot).map_err(|e| e.to_string())?;
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
    watch(
        &mut session,
        &args,
        &game,
        Some(&mut running),
        slot,
        names,
        Some(&save_dir),
    )
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

/// How long the watch hunts a party before naming its assumption: which slot
/// it is looking for, and that Enter chooses a different one.
const HINT_AFTER: Duration = Duration::from_secs(10);

/// Redraws the party until the emulator exits or the user stops the tool.
///
/// The emulator ending is not an error: both publishers' autoexecs end in
/// `exit`, so quitting the game closes DOSBox in every normal setup. This is
/// how sessions end. Every read failure stays fatal and non-zero, so a
/// permission error stays loud.
///
/// `save_dir` enables repicking the save slot: Enter at any point returns to
/// the slot question and the watch resumes with the new slot's names. `None`
/// (the `--pid` path) never prompts.
fn watch<R: squire_core::mem::Reader>(
    session: &mut Session<R>,
    args: &Args,
    game: &games::Game,
    mut running: Option<&mut Launched>,
    mut slot: char,
    mut names: Vec<String>,
    save_dir: Option<&Path>,
) -> Result<(), String> {
    let mut found_once = false;
    let mut searching_since = std::time::Instant::now();
    let mut hinted = false;
    // Once stdin hits end of file (a closed pipe), there is no keyboard to
    // listen to, and polling a fd at EOF would spin.
    let mut stdin_open = save_dir.is_some();
    eprintln!("gbs: waiting for the party of save slot {slot} to load...");

    loop {
        if let Some(r) = running.as_deref_mut() {
            if !r.is_running() {
                eprintln!("gbs: the emulator ended. Until next time.");
                return Ok(());
            }
        }

        match session.party() {
            Ok(party) => {
                if party.state == PartyState::NotFound && !found_once {
                    // Still waiting: the game is booting or sits in a menu.
                    // After a while, the missing party may mean a wrong pick,
                    // so name the assumption instead of hunting forever.
                    if !hinted && searching_since.elapsed() >= HINT_AFTER {
                        eprintln!(
                            "gbs: still looking for save slot {slot}'s party ({}). \
                             If another save is loaded, press Enter to choose a \
                             different slot.",
                            names.join(", ")
                        );
                        hinted = true;
                    }
                } else {
                    found_once = true;
                    if !args.json {
                        // Clear the screen and put the cursor at the top, so
                        // the table redraws in place.
                        print!("\x1b[2J\x1b[H");
                    }
                    print(&party, args);
                }
            }
            // The process went away between polls: the user quit the game.
            Err(Error::NoSuchProcess { .. }) => {
                eprintln!("gbs: the emulator ended. Until next time.");
                return Ok(());
            }
            Err(e) => return Err(e.to_string()),
        }

        let pause = if found_once {
            Duration::from_millis(args.interval_ms)
        } else {
            WAITING_POLL
        };

        // The pause doubles as the ear for Enter: wait on stdin readiness
        // with the poll cadence as the timeout, so a keypress is noticed at
        // once and no thread is added.
        if !stdin_open || !stdin_ready(pause) {
            continue;
        }
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => {
                stdin_open = false;
                continue;
            }
            Err(e) => return Err(format!("reading the keyboard: {e}")),
            Ok(_) => {}
        }
        let dir = save_dir.expect("stdin_open starts false without a save_dir");
        if let Some((new_slot, new_names)) = wizard::repick_slot(
            &mut std::io::stdin().lock(),
            &mut std::io::stderr(),
            game,
            dir,
        )? {
            slot = new_slot;
            names = new_names;
            session.retarget(names.clone());
            found_once = false;
            hinted = false;
            searching_since = std::time::Instant::now();
            eprintln!("gbs: waiting for the party of save slot {slot} to load...");
        }
    }
}

/// Waits up to `timeout` for stdin to have something to read.
///
/// The user types nothing on most polls, so this times out and the cadence is
/// exactly the sleep it replaced.
fn stdin_ready(timeout: Duration) -> bool {
    use std::os::fd::AsFd;
    let stdin = std::io::stdin();
    let mut fds = [nix::poll::PollFd::new(
        stdin.as_fd(),
        nix::poll::PollFlags::POLLIN,
    )];
    let ms = u16::try_from(timeout.as_millis()).unwrap_or(u16::MAX);
    match nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(ms)) {
        Ok(n) => n > 0,
        // A signal or a odd terminal is not worth dying over; keep polling.
        Err(_) => {
            std::thread::sleep(timeout);
            false
        }
    }
}

fn print(party: &squire_core::session::Party, args: &Args) {
    if args.json {
        println!("{}", output::json(party));
    } else {
        print!("{}", output::table(party));
    }
}

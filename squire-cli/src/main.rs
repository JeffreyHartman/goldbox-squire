//! `gbs` — the command line front end.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use squire_cli::args::{Args, USAGE};
use squire_cli::wizard;
use squire_cli::config::Config;
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

    // The command line wins over the config file, and what it says is then
    // remembered as a manual install, so a setting is given once rather than
    // every run.
    let mut config = Config::load();
    if config.remember_manual(&args) {
        if let Err(e) = config.save() {
            eprintln!("gbs: warning: could not save the settings: {e}");
        }
    }

    // Find the user's installs, once. A normal run reads the cached results;
    // the scan reruns only when there are none or a stored root vanished.
    if config.installs.is_empty() || config.needs_rediscovery() {
        let mut roots = squire_core::discover::default_roots();
        roots.extend(config.extra_roots.iter().map(PathBuf::from));
        if config.absorb(squire_core::discover::discover(&roots)) {
            if let Err(e) = config.save() {
                eprintln!("gbs: warning: could not save the settings: {e}");
            }
        }
        if config.installs.is_empty() {
            let searched: Vec<String> =
                roots.iter().map(|r| r.display().to_string()).collect();
            return Err(format!(
                "no game install found. Searched:\n  {}\n\
                 Point gbs at yours by hand once: \
                 gbs --conf /path/to/your.conf --game-dir /path/to/POOLRAD",
                searched.join("\n  ")
            ));
        }
    }

    let (key, slot) = wizard::choose(
        &mut std::io::stdin().lock(),
        &mut std::io::stderr(),
        &config,
        args.game.as_deref(),
        args.slot,
    )?;
    if config.last_install.as_ref() != Some(&key) {
        config.last_install = Some(key.clone());
        if let Err(e) = config.save() {
            eprintln!("gbs: warning: could not save the settings: {e}");
        }
    }
    let install = config.installs[&key].clone();
    let game_dir = install.save_dir();
    let names = saves::slot_party_names(&game_dir, slot).map_err(|e| e.to_string())?;
    let table = games::find(&install.game)
        .ok_or_else(|| format!("unknown game `{}` in the config", install.game))?
        .table;

    // Attaching to an emulator this tool did not start is the unusual path. It
    // needs a relaxed kernel.yama.ptrace_scope, so it is never the default.
    if let Some(pid) = args.pid {
        let mut session = Session::new(ProcessReader::new(pid), table, names);
        return watch(&mut session, &args, None, slot);
    }

    // The normal path. Starting the emulator is what makes the read permitted.
    let mut emulator = Emulator::new(install.emulator.as_deref().unwrap_or("dosbox"));
    for conf in &install.confs {
        emulator = emulator.conf(conf);
    }
    // Both publishers' autoexecs use relative mounts, so the emulator must
    // start in the folder holding the confs. For a manual install that is the
    // conf's own folder; the install root is the save folder there.
    emulator = emulator.current_dir(conf_dir(&install.confs, &install.root));
    let log = log_path();
    emulator = emulator.log_to(&log);

    let mut running = emulator.start().map_err(|e| e.to_string())?;
    eprintln!(
        "gbs: started the emulator as process {}. Its messages go to {}",
        running.pid(),
        log.display()
    );

    let mut session = Session::new(running.reader(), table, names);
    watch(&mut session, &args, Some(&mut running), slot)
}

/// The folder the emulator starts in: the one holding the confs.
fn conf_dir(confs: &[String], root: &str) -> PathBuf {
    match confs.first().map(Path::new) {
        Some(conf) if conf.is_absolute() => conf
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(root)),
        _ => PathBuf::from(root),
    }
}

/// Where the emulator's output goes, next to the config file.
fn log_path() -> PathBuf {
    Config::path()
        .and_then(|p| p.parent().map(|d| d.join("emulator.log")))
        .unwrap_or_else(|| std::env::temp_dir().join("gbs-emulator.log"))
}

/// Redraws the party until the emulator exits or the user stops the tool.
///
/// The emulator ending is not an error: both publishers' autoexecs end in
/// `exit`, so quitting the game closes DOSBox in every normal setup. This is
/// how sessions end. Every read failure stays fatal and non-zero, so a
/// permission error stays loud.
fn watch<R: squire_core::mem::Reader>(
    session: &mut Session<R>,
    args: &Args,
    mut running: Option<&mut Launched>,
    slot: char,
) -> Result<(), String> {
    let mut found_once = false;
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
        std::thread::sleep(pause);
    }
}

fn print(party: &squire_core::session::Party, args: &Args) {
    if args.json {
        println!("{}", output::json(party));
    } else {
        print!("{}", output::table(party));
    }
}

//! `gbs` — the command line front end.

use std::process::ExitCode;
use std::time::Duration;

use squire_cli::args::{Args, Mode, USAGE};
use squire_cli::config::Config;
use squire_cli::output;
use squire_core::launch::Emulator;
use squire_core::mem::ProcessReader;
use squire_core::session::Session;
use squire_core::{saves, table::Table};

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

    if args.mode == Mode::Help {
        print!("{USAGE}");
        return Ok(());
    }

    // The command line wins over the config file, and what it says is then
    // remembered, so that a setting is given once rather than every run.
    let mut config = Config::load();
    if config.merge(&args) {
        if let Err(e) = config.save() {
            eprintln!("gbs: warning: could not save the settings: {e}");
        }
    }

    let game_dir = config
        .game_dir
        .clone()
        .ok_or("no game folder set. Run `gbs --game-dir /path/to/POOLRAD` once.")?;

    let names = saves::party_names(&game_dir).map_err(|e| e.to_string())?;
    let table = Table::pool_of_radiance();

    // Attaching to an emulator this tool did not start is the unusual path. It
    // needs a relaxed kernel.yama.ptrace_scope, so it is never the default.
    if let Some(pid) = args.pid {
        let mut session = Session::new(ProcessReader::new(pid), table, names);
        return report(&mut session, &args);
    }

    // The normal path. Starting the emulator is what makes the read permitted.
    let mut emulator = Emulator::new(config.dosbox.as_deref().unwrap_or("dosbox"));
    if let Some(conf) = &config.conf {
        emulator = emulator.arg("-conf").arg(conf);
    }
    let mut running = emulator.start().map_err(|e| e.to_string())?;
    eprintln!("gbs: started the emulator as process {}", running.pid());

    let mut session = Session::new(running.reader(), table, names);
    let result = report(&mut session, &args);

    // The emulator is deliberately left running. The player keeps playing when
    // this tool exits.
    if !running.is_running() {
        eprintln!("gbs: the emulator ended.");
    }
    result
}

fn report<R: squire_core::mem::Reader>(
    session: &mut Session<R>,
    args: &Args,
) -> Result<(), String> {
    match args.mode {
        Mode::Once => {
            let party = session.party().map_err(|e| e.to_string())?;
            print(&party, args);
            Ok(())
        }
        Mode::Watch => {
            loop {
                match session.party() {
                    Ok(party) => {
                        if !args.json {
                            // Clear the screen and put the cursor at the top,
                            // so the table redraws in place.
                            print!("\x1b[2J\x1b[H");
                        }
                        print(&party, args);
                    }
                    // A read that fails is a failure, and the exit status
                    // must say so. A script that watches gbs cannot tell a
                    // clean stop from a permission error otherwise.
                    Err(e) => return Err(e.to_string()),
                }
                std::thread::sleep(Duration::from_millis(args.interval_ms));
            }
        }
        Mode::Help => Ok(()),
    }
}

fn print(party: &squire_core::session::Party, args: &Args) {
    if args.json {
        println!("{}", output::json(party));
    } else {
        print!("{}", output::table(party));
    }
}

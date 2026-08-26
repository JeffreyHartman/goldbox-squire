//! The watch loop: poll the session, draw the party, listen for the user.
//!
//! The loop makes no decision about how anything looks. It hands a party to a
//! [`Screen`] and asks a [`Keys`] to wait out the interval, and both of those
//! are traits so that a second front end is a second implementation rather
//! than a second copy of this file.

use std::time::{Duration, Instant};

use squire_core::mem::Reader;
use squire_core::session::{Party, PartyState, Session};
use squire_core::Error;

/// Where the party is shown.
///
/// The printed table is one implementation. The HUD is another. Nothing here
/// says terminal, escape sequence, or column.
pub trait Screen {
    /// Fresh numbers, once per poll, for as long as the party is in memory.
    fn party(&mut self, party: &Party);

    /// News for the user: what is being waited for, and why the watch ended.
    /// The message is a sentence with no program name on the front, because
    /// where it goes decides how it is labelled.
    fn notice(&mut self, message: &str);
}

/// What the user did during the pause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interrupt {
    /// Nothing. Poll again.
    None,
    /// A different save slot was chosen, with the names to look for.
    Repick { slot: char, names: Vec<String> },
}

/// The pause between polls, and the user's chance to interrupt it.
///
/// The pause and the keyboard are one seam because they are one wait: the
/// loop blocks on the keyboard for as long as the interval lasts, so no
/// thread is added and a keypress is noticed at once. A front end that reads
/// keys differently replaces both together.
pub trait Keys {
    fn wait(&mut self, pause: Duration) -> Result<Interrupt, String>;
}

/// Both ways a watch ends without an error. The handle says the process went,
/// or a read finds it gone; the user is told the same thing either way.
const ENDED: &str = "the emulator ended. Until next time.";

/// The emulator, as much of it as the loop needs.
pub trait Alive {
    fn is_running(&mut self) -> bool;
}

impl Alive for squire_core::launch::Launched {
    fn is_running(&mut self) -> bool {
        squire_core::launch::Launched::is_running(self)
    }
}

/// How long the loop waits, and how long it hunts before it explains itself.
#[derive(Debug, Clone, Copy)]
pub struct Watch {
    /// Between redraws, once a party was found.
    pub interval: Duration,
    /// Between polls while no party was found yet. Each failed poll is a full
    /// memory sweep through DOS boot, title screen and load menu, so this is
    /// far slower than the redraw interval.
    pub waiting_poll: Duration,
    /// How long the watch hunts before naming its assumption: which slot it
    /// is looking for, and that Enter chooses a different one.
    pub hint_after: Duration,
}

impl Default for Watch {
    fn default() -> Self {
        Watch {
            interval: Duration::from_millis(500),
            waiting_poll: Duration::from_secs(2),
            hint_after: Duration::from_secs(10),
        }
    }
}

/// Draws the party until the emulator exits or the user stops the tool.
///
/// The emulator ending is not an error: both publishers' autoexecs end in
/// `exit`, so quitting the game closes DOSBox in every normal setup. This is
/// how sessions end. Every other read failure stays fatal and non-zero, so a
/// permission error stays loud.
///
/// `running` is the emulator this tool started. The `--pid` path has no handle
/// to one, and passes `None`; there the loop learns the process is gone from
/// the next read that fails.
pub fn watch<R: Reader>(
    session: &mut Session<R>,
    timing: &Watch,
    screen: &mut dyn Screen,
    keys: &mut dyn Keys,
    mut running: Option<&mut dyn Alive>,
    mut slot: Option<char>,
    mut names: Vec<String>,
) -> Result<(), String> {
    let mut found_once = false;
    let mut searching_since = Instant::now();
    let mut hinted = false;
    screen.notice(&waiting_for(slot));

    loop {
        if let Some(r) = running.as_deref_mut() {
            if !r.is_running() {
                screen.notice(ENDED);
                return Ok(());
            }
        }

        match session.party() {
            Ok(party) => {
                if party.state == PartyState::NotFound && !found_once {
                    // Still waiting: the game is booting or sits in a menu.
                    // After a while, the missing party may mean a wrong pick,
                    // so name the assumption instead of hunting forever. With
                    // no slot chosen there is no assumption to name.
                    if let Some(letter) = slot {
                        if !hinted && searching_since.elapsed() >= timing.hint_after {
                            screen.notice(&format!(
                                "still looking for save slot {letter}'s party ({}). \
                                 If another save is loaded, press Enter to choose a \
                                 different slot.",
                                names.join(", ")
                            ));
                            hinted = true;
                        }
                    }
                } else {
                    found_once = true;
                    screen.party(&party);
                }
            }
            // The process went away between polls: the user quit the game.
            Err(Error::NoSuchProcess { .. }) => {
                screen.notice(ENDED);
                return Ok(());
            }
            Err(e) => return Err(e.to_string()),
        }

        let pause = if found_once {
            timing.interval
        } else {
            timing.waiting_poll
        };

        if let Interrupt::Repick {
            slot: new_slot,
            names: new_names,
        } = keys.wait(pause)?
        {
            slot = Some(new_slot);
            names = new_names;
            session.retarget(names.clone());
            found_once = false;
            hinted = false;
            searching_since = Instant::now();
            screen.notice(&waiting_for(slot));
        }
    }
}

/// What the watch says it is doing, before anything has been found.
fn waiting_for(slot: Option<char>) -> String {
    match slot {
        Some(letter) => format!("waiting for the party of save slot {letter} to load..."),
        None => "no saved game yet. Play on; after you save in game, press Enter here to pick it."
            .to_string(),
    }
}

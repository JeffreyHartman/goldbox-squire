//! The real keyboard: the pause between polls, with an ear on standard input.
//!
//! Enter is the only key the printed front end reads, and it means "let me
//! pick a different save slot". Waiting for it is also how the loop spends its
//! interval, so this one type owns both.

use std::path::{Path, PathBuf};
use std::time::Duration;

use squire_core::games;

use crate::watch::{Interrupt, Keys};
use crate::wizard;

/// Waits on standard input, and asks the slot question when Enter arrives.
pub struct Stdin {
    game: games::Game,
    /// The install's own save folder, which is where repicking starts looking.
    /// `None` on the `--pid` path: that run never started a game, so it has no
    /// install to offer.
    save_dir: Option<PathBuf>,
    /// Cleared at end of file. A closed pipe has no keyboard behind it, and
    /// polling a descriptor at end of file would spin.
    listening: bool,
}

impl Stdin {
    pub fn new(game: &games::Game, save_dir: Option<&Path>) -> Self {
        Stdin {
            game: game.clone(),
            save_dir: save_dir.map(Path::to_path_buf),
            listening: save_dir.is_some(),
        }
    }
}

impl Keys for Stdin {
    fn wait(&mut self, pause: Duration) -> Result<Interrupt, String> {
        // Nobody is typing: spend the interval asleep. Returning at once would
        // turn the poll cadence into a spin.
        if !self.listening {
            std::thread::sleep(pause);
            return Ok(Interrupt::None);
        }
        if !ready(pause) {
            return Ok(Interrupt::None);
        }
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => {
                self.listening = false;
                return Ok(Interrupt::None);
            }
            Err(e) => return Err(format!("reading the keyboard: {e}")),
            Ok(_) => {}
        }
        let dir = self
            .save_dir
            .as_deref()
            .expect("listening starts false without a save folder");
        let picked = wizard::repick(
            &mut std::io::stdin().lock(),
            &mut std::io::stderr(),
            &self.game,
            dir,
        )?;
        Ok(match picked {
            Some((slot, names)) => Interrupt::Repick { slot, names },
            None => Interrupt::None,
        })
    }
}

/// Waits up to `timeout` for standard input to have something to read.
///
/// The user types nothing on most polls, so this times out and the cadence is
/// exactly the sleep it replaced.
fn ready(timeout: Duration) -> bool {
    use std::os::fd::AsFd;
    let stdin = std::io::stdin();
    let mut fds = [nix::poll::PollFd::new(
        stdin.as_fd(),
        nix::poll::PollFlags::POLLIN,
    )];
    let ms = u16::try_from(timeout.as_millis()).unwrap_or(u16::MAX);
    match nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(ms)) {
        Ok(n) => n > 0,
        // A signal or an odd terminal is not worth dying over; keep polling.
        Err(_) => {
            std::thread::sleep(timeout);
            false
        }
    }
}

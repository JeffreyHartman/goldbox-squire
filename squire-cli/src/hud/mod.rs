//! The HUD: the live party on a screen you glance at, rather than a table
//! reprinted into your scrollback.
//!
//! Two seams of the watch loop meet here. [`crate::watch::Screen`] is where
//! the party arrives and [`crate::watch::Keys`] is the pause and the ear for
//! the keyboard, and the HUD is one thing wearing both, because a keypress
//! changes what is drawn and a redraw has to happen inside the pause. They
//! share one [`Inner`] through an `Rc<RefCell<_>>` for exactly that reason and
//! for no other.
//!
//! This file owns the terminal and nothing else. What the HUD knows is
//! [`View`], which has no terminal in it and is where the keyboard contract is
//! tested. What is shown is [`crate::layout`]'s decision, and [`draw`] draws
//! its answer.

pub mod draw;
pub mod theme;
pub mod view;

use std::cell::RefCell;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use squire_core::games;
use squire_core::session::Party;

use crate::layout::{Caption, Size};
use crate::watch::{Interrupt, Keys, Screen};
use crate::wizard;

pub use view::{Press, View};

/// The terminal, and what is drawn on it.
struct Inner {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    view: View,
    /// The size of the last draw, so that the run can remember where the user
    /// put the window.
    size: Size,
}

impl Inner {
    fn redraw(&mut self) -> Result<(), String> {
        let area = self
            .terminal
            .size()
            .map_err(|e| format!("asking the terminal its size: {e}"))?;
        self.size = Size {
            cols: area.width,
            rows: area.height,
        };
        let plan = self.view.plan(self.size);
        let party = self.view.party();
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                draw::draw(area, frame.buffer_mut(), &plan, &party);
            })
            .map_err(|e| format!("drawing the party: {e}"))?;
        Ok(())
    }
}

/// Puts the terminal back the way it was found.
///
/// On `Drop` so that it happens after an error and after a panic alike. A tool
/// that leaves a terminal in raw mode has broken the shell the user goes back
/// to, and no error message is worth that.
impl Drop for Inner {
    fn drop(&mut self) {
        restore();
    }
}

fn restore() {
    let _ = disable_raw_mode();
    let _ = io::stdout().execute(LeaveAlternateScreen);
    let _ = io::stdout().execute(crossterm::cursor::Show);
    let _ = io::stdout().flush();
}

/// The HUD, and the two handles the watch loop takes.
///
/// Held by the caller for as long as the watch runs. Dropping it restores the
/// terminal.
pub struct Hud {
    inner: Rc<RefCell<Inner>>,
}

impl Hud {
    /// Takes over the terminal and draws the first, empty, frame.
    ///
    /// Every failure between raw mode going on and `Inner` existing undoes the
    /// takeover by hand: there is no value yet for `Drop` to hang from, and
    /// returning an error into a shell with no echo is the worst outcome this
    /// function has.
    pub fn start(caption: Caption, remembered: Option<Size>) -> Result<Hud, String> {
        enable_raw_mode().map_err(|e| format!("putting the terminal in raw mode: {e}"))?;
        let mut out = io::stdout();
        if let Err(e) = execute!(out, EnterAlternateScreen, crossterm::cursor::Hide) {
            restore();
            return Err(format!("taking over the terminal: {e}"));
        }

        // A panic past this point would otherwise leave the shell in raw mode
        // with no echo. The old hook still runs, so the message is not lost.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));

        let terminal = match Terminal::new(CrosstermBackend::new(out)) {
            Ok(terminal) => terminal,
            Err(e) => {
                restore();
                return Err(format!("starting the interface: {e}"));
            }
        };
        let inner = Inner {
            terminal,
            view: View::new(caption),
            // Replaced by the first draw. The remembered size is what the
            // window was asked for, not what it got, so it is never trusted
            // as the size to draw at.
            size: remembered.unwrap_or(Size { cols: 0, rows: 0 }),
        };
        let hud = Hud {
            inner: Rc::new(RefCell::new(inner)),
        };
        hud.inner.borrow_mut().redraw()?;
        Ok(hud)
    }

    /// The drawing seam of the watch loop.
    pub fn screen(&self) -> HudScreen {
        HudScreen {
            inner: Rc::clone(&self.inner),
        }
    }

    /// The pause and the keyboard seam of the watch loop.
    ///
    /// `save_dir` is the install's own save folder, which is where the slot
    /// repick starts looking. `None` on the `--pid` path, which never started
    /// a game and so has no install to offer.
    pub fn keys(&self, game: &games::Game, save_dir: Option<&Path>) -> HudKeys {
        HudKeys {
            inner: Rc::clone(&self.inner),
            game: game.clone(),
            save_dir: save_dir.map(Path::to_path_buf),
        }
    }

    /// Redraws if the window changed size since the last frame.
    ///
    /// A view spends its time waiting on a socket rather than on a keyboard,
    /// so nothing else would notice a drag until the next poll, and a window
    /// that does not reflow while it is being resized looks like a hang.
    pub fn pump_resizes(&self) -> Result<(), String> {
        while event::poll(Duration::ZERO).map_err(|e| format!("waiting on the terminal: {e}"))? {
            let event = event::read().map_err(|e| format!("reading the terminal: {e}"))?;
            if matches!(event, Event::Resize(..)) {
                self.inner.borrow_mut().redraw()?;
            }
        }
        Ok(())
    }

    /// The size of the last frame drawn, for the config to remember.
    pub fn size(&self) -> Size {
        self.inner.borrow().size
    }
}

/// Where the party is drawn.
pub struct HudScreen {
    inner: Rc<RefCell<Inner>>,
}

impl Screen for HudScreen {
    fn party(&mut self, party: &Party) {
        let mut inner = self.inner.borrow_mut();
        inner.view.saw(party);
        // A failed draw is not worth ending a run over: the next poll draws
        // again, and the loop is what notices a terminal that has really gone.
        let _ = inner.redraw();
    }

    fn notice(&mut self, message: &str) {
        let mut inner = self.inner.borrow_mut();
        inner.view.note(message);
        let _ = inner.redraw();
    }
}

/// The pause between polls, with an ear on the keyboard.
pub struct HudKeys {
    inner: Rc<RefCell<Inner>>,
    game: games::Game,
    save_dir: Option<PathBuf>,
}

impl Keys for HudKeys {
    fn wait(&mut self, pause: Duration) -> Result<Interrupt, String> {
        let deadline = Instant::now() + pause;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let ready = event::poll(left).map_err(|e| format!("waiting on the keyboard: {e}"))?;
            if !ready {
                return Ok(Interrupt::None);
            }
            let event = event::read().map_err(|e| format!("reading the keyboard: {e}"))?;
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let press = {
                        let mut inner = self.inner.borrow_mut();
                        let press = inner.view.press(key.code, key.modifiers);
                        let _ = inner.redraw();
                        press
                    };
                    match press {
                        Press::Quit => return Ok(Interrupt::Quit),
                        Press::AskForSlot => return self.repick(),
                        Press::Handled => {}
                    }
                }
                // The window changed size. Reflow now rather than at the next
                // poll, or a drag looks like a hang.
                Event::Resize(..) => {
                    let _ = self.inner.borrow_mut().redraw();
                }
                _ => {}
            }
            if Instant::now() >= deadline {
                return Ok(Interrupt::None);
            }
        }
    }
}

impl HudKeys {
    /// The slot repick, which is the one place the HUD could quietly lose a
    /// feature the printed front end had.
    ///
    /// The wizard asks its question on a normal terminal: it prints a menu and
    /// reads a line, and neither works in raw mode. So the HUD steps aside for
    /// the length of the question and takes the terminal back afterwards. That
    /// is far less code than a second copy of the menu drawn on screen, and it
    /// keeps one wizard rather than two that can disagree.
    fn repick(&mut self) -> Result<Interrupt, String> {
        let Some(dir) = self.save_dir.clone() else {
            return Ok(Interrupt::None);
        };
        restore();
        let picked = wizard::repick(&mut io::stdin().lock(), &mut io::stderr(), &self.game, &dir);
        // The terminal comes back whatever the wizard did, including when it
        // failed, or the error message lands on a screen nobody can read.
        let resumed = resume();
        let mut inner = self.inner.borrow_mut();
        if let Ok(Some((slot, _))) = &picked {
            inner.view.retargeted(*slot);
        }
        let _ = inner.terminal.clear();
        let _ = inner.redraw();
        drop(inner);
        resumed?;
        Ok(match picked? {
            Some((slot, names)) => Interrupt::Repick { slot, names },
            None => Interrupt::None,
        })
    }
}

/// Takes the terminal back after the wizard has had it.
fn resume() -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("putting the terminal back in raw mode: {e}"))?;
    execute!(io::stdout(), EnterAlternateScreen, crossterm::cursor::Hide)
        .map_err(|e| format!("taking the terminal back: {e}"))
}

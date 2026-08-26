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
//! Nothing here decides what is shown. That is [`crate::layout`]'s job, and
//! [`draw`] draws its answer.

pub mod draw;
pub mod theme;

use std::cell::RefCell;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use squire_core::games;
use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

use crate::layout::{self, Sitting, Size, Toggles};
use crate::watch::{Interrupt, Keys, Screen};
use crate::wizard;

/// The panels the number keys are reserved for. Only the first exists; the
/// rest are a promise that the second one will not need a menu built for it.
const PANELS: [&str; 1] = ["party"];

/// Everything the HUD holds between polls.
struct Inner {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// The last characters read. Kept when the anchor is lost, because a
    /// rescan takes a moment and the last known values are still worth
    /// something. They are shown dimmed, never as live.
    characters: Vec<Character>,
    state: PartyState,
    sitting: Sitting,
    toggles: Toggles,
    /// Which character the highlight sits on.
    selected: usize,
    /// The size of the last draw, so that the run can remember where the
    /// user put the window.
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
        let party = Party {
            state: self.state,
            characters: self.characters.clone(),
        };
        let plan = layout::plan(self.size, &party, &self.sitting, self.toggles);
        let selected = self.selected;
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                draw::draw(area, frame.buffer_mut(), &plan, &party, selected);
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
    pub fn start(sitting: Sitting, remembered: Option<Size>) -> Result<Hud, String> {
        enable_raw_mode().map_err(|e| format!("putting the terminal in raw mode: {e}"))?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)
            .map_err(|e| format!("taking over the terminal: {e}"))?;

        // A panic past this point would otherwise leave the shell in raw mode
        // with no echo. The old hook still runs, so the message is not lost.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));

        let terminal = Terminal::new(CrosstermBackend::new(out))
            .map_err(|e| format!("starting the interface: {e}"))?;
        let inner = Inner {
            terminal,
            characters: Vec::new(),
            state: PartyState::NotFound,
            sitting,
            toggles: Toggles::default(),
            selected: 0,
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
        inner.state = party.state;
        if !party.characters.is_empty() {
            inner.characters = party.characters.clone();
            // The loop's last word was about finding the party. It has been
            // found, so the words go and the status line speaks for itself.
            inner.sitting.note = None;
        }
        if inner.selected >= inner.characters.len() {
            inner.selected = inner.characters.len().saturating_sub(1);
        }
        // A failed draw is not worth ending a run over: the next poll draws
        // again, and the loop is what notices a terminal that has really gone.
        let _ = inner.redraw();
    }

    fn notice(&mut self, message: &str) {
        let mut inner = self.inner.borrow_mut();
        inner.sitting.note = Some(message.to_string());
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
                    if let Some(interrupt) = self.press(key.code, key.modifiers)? {
                        return Ok(interrupt);
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
    /// One keypress. `None` means it was handled and the pause carries on.
    fn press(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<Interrupt>, String> {
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return Ok(Some(Interrupt::Quit));
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(Interrupt::Quit)),
            KeyCode::Enter => return self.repick().map(Some),
            KeyCode::Up | KeyCode::Char('k') => self.move_highlight(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_highlight(1),
            KeyCode::Char('a') => {
                let mut inner = self.inner.borrow_mut();
                inner.toggles.abilities = !inner.toggles.abilities;
                let _ = inner.redraw();
            }
            KeyCode::Char('c') => self.cycle_columns(),
            KeyCode::Char(digit @ '1'..='9') => {
                // Reserved. Only one panel exists, so every other number is a
                // key that has not grown its screen yet.
                let wanted = usize::from(digit as u8 - b'1');
                if let Some(panel) = PANELS.get(wanted) {
                    let mut inner = self.inner.borrow_mut();
                    inner.sitting.panel = (*panel).to_string();
                    let _ = inner.redraw();
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Moves the highlight, and stops at each end rather than wrapping. A HUD
    /// glanced at sideways should not move the highlight somewhere surprising.
    fn move_highlight(&mut self, by: i32) {
        let mut inner = self.inner.borrow_mut();
        let last = inner.characters.len().saturating_sub(1);
        let next = i64::from(by) + inner.selected as i64;
        inner.selected = next.clamp(0, last as i64) as usize;
        let _ = inner.redraw();
    }

    /// Steps through the arrangements the party divides into evenly, and then
    /// back to letting the rule decide.
    ///
    /// This is what keeps both of 034's answers for a short wide window: the
    /// rule picks three across, and this asks for six.
    fn cycle_columns(&mut self) {
        let mut inner = self.inner.borrow_mut();
        let n = u16::try_from(inner.characters.len()).unwrap_or(0);
        let even: Vec<u16> = (1..=n).filter(|across| n % across == 0).collect();
        inner.toggles.across = match inner.toggles.across {
            None => even.first().copied(),
            Some(current) => even.iter().copied().find(|a| *a > current),
        };
        let _ = inner.redraw();
    }

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
        let _ = inner.terminal.clear();
        let _ = inner.redraw();
        drop(inner);
        resumed?;
        Ok(match picked? {
            Some((slot, names)) => {
                let mut inner = self.inner.borrow_mut();
                inner.sitting.slot = Some(slot);
                inner.characters.clear();
                inner.selected = 0;
                Interrupt::Repick { slot, names }
            }
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

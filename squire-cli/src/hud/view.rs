//! What the HUD knows, with no terminal in it.
//!
//! Everything a keypress or a poll changes lives here, and none of it needs a
//! screen to test. The terminal is one layer out, in [`super`], which does
//! nothing but hand this a size and draw what it answers.
//!
//! Splitting it this way is what makes the keyboard contract testable at all.
//! A key that moves the highlight past the end of the party, or a repick that
//! leaves the highlight pointing at a character who is gone, is a state
//! change, and a state change should not need a pseudo-terminal to check.

use crossterm::event::{KeyCode, KeyModifiers};

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

use crate::layout::{self, Caption, Plan, Size, Toggles};

/// The panels the number keys are reserved for. Only the first exists; the
/// rest are a promise that the second one will not need a menu built for it.
const PANELS: [&str; 1] = ["party"];

/// What a keypress asked the run to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The view changed, or the key meant nothing. Either way, redraw.
    Handled,
    /// Stop the run.
    Quit,
    /// Ask the wizard for a different save slot. Doing it means giving the
    /// terminal back for the length of the question, which is [`super`]'s job.
    AskForSlot,
}

/// The HUD's state between polls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    /// The last characters read. Kept when the anchor is lost, because a
    /// rescan takes a moment and the last known values are still worth
    /// something. They are shown dimmed, never as live.
    characters: Vec<Character>,
    state: PartyState,
    caption: Caption,
    toggles: Toggles,
    /// Which character the highlight sits on.
    selected: usize,
}

impl View {
    pub fn new(caption: Caption) -> View {
        View {
            characters: Vec::new(),
            state: PartyState::NotFound,
            caption,
            toggles: Toggles::default(),
            selected: 0,
        }
    }

    /// The party as the screen should show it: the current state, and the last
    /// characters read.
    ///
    /// The session returns nothing at all when the anchor is gone, so without
    /// this the screen would empty rather than dim.
    pub fn party(&self) -> Party {
        Party {
            state: self.state,
            characters: self.characters.clone(),
        }
    }

    pub fn plan(&self, size: Size) -> Plan {
        layout::plan(size, &self.party(), &self.caption, self.toggles)
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    /// A fresh poll.
    pub fn saw(&mut self, party: &Party) {
        self.state = party.state;
        if !party.characters.is_empty() {
            self.characters = party.characters.clone();
            // The loop's last word was about finding the party. It has been
            // found, so the words go and the status line speaks for itself.
            self.caption.note = None;
        }
        self.clamp_highlight();
    }

    /// The watch loop's latest word.
    pub fn note(&mut self, message: &str) {
        self.caption.note = Some(message.to_string());
    }

    /// The user picked a different save slot. Everything on screen belonged to
    /// the old one.
    pub fn retargeted(&mut self, slot: char) {
        self.caption.slot = Some(slot);
        self.characters.clear();
        self.state = PartyState::NotFound;
        self.selected = 0;
    }

    /// One keypress.
    pub fn press(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Press {
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return Press::Quit;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Press::Quit,
            KeyCode::Enter => return Press::AskForSlot,
            KeyCode::Up | KeyCode::Char('k') => self.move_highlight(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_highlight(1),
            KeyCode::Char('a') => self.toggles.abilities = !self.toggles.abilities,
            KeyCode::Char('c') => self.cycle_columns(),
            KeyCode::Char(digit @ '1'..='9') => {
                // Reserved. Only one panel exists, so every other number is a
                // key that has not grown its screen yet.
                let wanted = usize::from(digit as u8 - b'1');
                if let Some(panel) = PANELS.get(wanted) {
                    self.caption.panel = (*panel).to_string();
                }
            }
            _ => {}
        }
        Press::Handled
    }

    /// Moves the highlight, and stops at each end rather than wrapping. A HUD
    /// glanced at sideways should not move the highlight somewhere surprising.
    fn move_highlight(&mut self, by: i64) {
        let last = self.characters.len().saturating_sub(1) as i64;
        self.selected = (self.selected as i64 + by).clamp(0, last) as usize;
    }

    fn clamp_highlight(&mut self) {
        if self.selected >= self.characters.len() {
            self.selected = self.characters.len().saturating_sub(1);
        }
    }

    /// Steps through the arrangements the party divides into evenly, and then
    /// back to letting the rule decide.
    ///
    /// This is what keeps both of 034's answers for a short wide window: the
    /// rule picks three across, and this asks for six. The list is the layout
    /// plan's own, so the key can never ask for an arrangement the rule would
    /// not have offered.
    fn cycle_columns(&mut self) {
        let n = u16::try_from(self.characters.len()).unwrap_or(0);
        let even = layout::arrangements(n);
        self.toggles.across = match self.toggles.across {
            None => even.first().copied(),
            Some(current) => even.into_iter().find(|across| *across > current),
        };
    }
}

//! What the HUD knows, with no terminal in it.
//!
//! Everything a keypress or a poll changes lives here, and none of it needs a
//! screen to test. The terminal is one layer out, in [`super`], which does
//! nothing but hand this a size and draw what it answers.
//!
//! Splitting it this way is what makes the keyboard contract testable at all.
//! A toggle, a panel, or a repick clearing the party the old slot left behind
//! is a state change, and a state change should not need a pseudo-terminal to
//! check.

use crossterm::event::{KeyCode, KeyModifiers};

use squire_core::record::Character;
use squire_core::session::{Party, PartyState};

use crate::layout::{self, Axis, Caption, Plan, Size, Toggles};

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
}

impl View {
    pub fn new(caption: Caption) -> View {
        View {
            characters: Vec::new(),
            state: PartyState::NotFound,
            caption,
            toggles: Toggles::default(),
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

    /// A fresh poll.
    pub fn saw(&mut self, party: &Party) {
        self.state = party.state;
        if !party.characters.is_empty() {
            self.characters = party.characters.clone();
            // The loop's last word was about finding the party. It has been
            // found, so the words go and the status line speaks for itself.
            self.caption.note = None;
        }
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
    }

    /// One keypress.
    pub fn press(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Press {
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return Press::Quit;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Press::Quit,
            // Not Enter. Enter is the key you press to find out what a key
            // does, and going back to the wizard is not a thing to discover by
            // accident. `s` for slot, and Enter means nothing.
            KeyCode::Char('s') => return Press::AskForSlot,
            KeyCode::Char('a') => self.toggles.abilities = !self.toggles.abilities,
            KeyCode::Char('c') => self.flip_axis(),
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

    fn flip_axis(&mut self) {
        self.toggles.axis = match self.toggles.axis {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        };
    }
}

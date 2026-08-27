//! A view: one window, drawing what the host sends it.
//!
//! A view never reads the emulator and never learns an address. It opens the
//! host's socket, draws the party that arrives on it, and sends the user's
//! decisions back. Only the host may read DOSBox, because only DOSBox's
//! parent may, and there is only one of those. ADR 0005 has the argument.
//!
//! This module is the socket turned into draw calls. What is drawn is
//! [`crate::hud`]'s business, and it is the same HUD whether it was started
//! here or by `--plain`'s absence in a single-window run.

use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use squire_core::games;
use squire_core::session::Party;

use crate::hud::Hud;
use crate::layout::{Caption, Size};
use crate::terminals::ViewKind;
use crate::watch::Screen;
use crate::wire::{self, Hello};

/// One message from the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoming {
    Hello(Hello),
    Party(Party),
    Notice(String),
    /// A message this build has no use for. Ignoring it is what lets the wire
    /// grow a message without every view being rebuilt for it.
    Unknown,
}

/// One line from the host.
///
/// An error here is a message that claimed to be something and was not, which
/// is worth saying out loud. A message this build does not know is not an
/// error, and neither is a blank line.
pub fn decode(line: &str) -> Result<Incoming, String> {
    if line.trim().is_empty() {
        return Ok(Incoming::Unknown);
    }
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| format!("the host sent a line that is not JSON: {e}"))?;
    match value["kind"].as_str() {
        Some("hello") => Ok(Incoming::Hello(Hello::from_value(&value)?)),
        Some("party") => Ok(Incoming::Party(wire::party_from_value(&value["party"])?)),
        Some("notice") => Ok(Incoming::Notice(
            value["message"].as_str().unwrap_or_default().to_string(),
        )),
        _ => Ok(Incoming::Unknown),
    }
}

/// Opens the host's socket.
///
/// The path is in the message because the usual way to see this is a stale
/// one: a window reopened by hand after the run it belonged to ended.
pub fn connect(socket: &Path) -> Result<UnixStream, String> {
    UnixStream::connect(socket).map_err(|e| {
        format!(
            "no run is listening on {}: {e}. The socket belongs to a running gbs, \
             and it goes when the run does.",
            socket.display()
        )
    })
}

/// Draws the run listening on `socket` until it ends or the user stops it.
///
/// `remembered` is the size the window was left at last time, used only to
/// seed the HUD; the first draw replaces it with the size the window really
/// got, because the terminal has the last word on that. The size it was left
/// at comes back, because the view is the process that knows it now.
pub fn run(kind: ViewKind, socket: &Path, remembered: Option<Size>) -> Result<Size, String> {
    let stream = connect(socket)?;
    let mut lines = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| format!("listening to the host: {e}"))?,
    );

    let hello = first_hello(&mut lines)?;
    // Named here rather than used, because a view that cannot name the game
    // is a view drawing a party it may be decoding with the wrong table.
    games::find(&hello.game_id).ok_or_else(|| {
        format!(
            "the host is watching `{}`, which this build of gbs does not know",
            hello.game_id
        )
    })?;

    let interface = Hud::start(
        Caption {
            game: hello.game_name.clone(),
            slot: hello.slot,
            panel: kind.as_str().to_string(),
            note: None,
        },
        remembered,
    )?;
    let mut screen = interface.screen();

    // Blocking reads from here on. A view has nothing to do between messages
    // except redraw on a resize, and the host sends a party every poll, so
    // the longest this ever waits is one poll interval.
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|e| format!("listening to the host: {e}"))?;

    let outcome = draw_until_the_host_goes(&mut lines, &interface, &mut screen);
    let size = interface.size();
    drop(interface);
    outcome.map(|()| size)
}

/// Reads messages and draws them, until the host closes the socket.
///
/// The host going is how a run ends, so it is not an error. The user quit the
/// game, or quit gbs, and either way the window has nothing left to show.
fn draw_until_the_host_goes(
    lines: &mut BufReader<UnixStream>,
    interface: &Hud,
    screen: &mut dyn Screen,
) -> Result<(), String> {
    let mut line = String::new();
    loop {
        line.clear();
        match lines.read_line(&mut line) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(e) if would_block(&e) => {
                // Nothing arrived this slice. Spend it noticing a resize, so
                // that dragging the window edge reflows rather than looking
                // like a hang until the next poll.
                interface.pump_resizes()?;
                continue;
            }
            Err(e) => return Err(format!("listening to the host: {e}")),
        }
        match decode(&line) {
            Ok(Incoming::Party(party)) => screen.party(&party),
            Ok(Incoming::Notice(message)) => screen.notice(&message),
            // A second hello would mean a second host on one socket.
            Ok(Incoming::Hello(_)) | Ok(Incoming::Unknown) => {}
            // One unreadable message is not worth closing the window over.
            // The next poll sends the party again.
            Err(problem) => screen.notice(&problem),
        }
        interface.pump_resizes()?;
    }
}

/// The host's first word, which says which run this window belongs to.
fn first_hello(lines: &mut BufReader<UnixStream>) -> Result<Hello, String> {
    let mut line = String::new();
    lines
        .read_line(&mut line)
        .map_err(|e| format!("listening to the host: {e}"))?;
    match decode(&line)? {
        Incoming::Hello(hello) => Ok(hello),
        _ => Err("the host did not say which run this is".into()),
    }
}

/// A read that timed out rather than failed.
fn would_block(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::Interrupted
    )
}

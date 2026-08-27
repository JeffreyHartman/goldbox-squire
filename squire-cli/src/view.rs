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

use std::io::{BufRead, BufReader, Write};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use squire_core::games;
use squire_core::session::Party;

use crate::hud::Hud;
use crate::layout::{Caption, Size};
use crate::terminals::ViewKind;
use crate::watch::{Interrupt, Keys, Screen};
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
    // The game is what the slot question is asked about, so a view that
    // cannot name it is a view whose Enter key does nothing.
    let game = games::find(&hello.game_id).ok_or_else(|| {
        format!(
            "the host is watching `{}`, which this build of gbs does not know",
            hello.game_id
        )
    })?;

    // One kind today. The match is here rather than an `if`, so that adding
    // the map view is a new arm the compiler asks for rather than a branch
    // somebody has to remember to write.
    match kind {
        ViewKind::Hud => {}
    }

    let interface = Hud::start(
        Caption {
            game: hello.game_name.clone(),
            slot: hello.slot,
            // The panel, not the view kind. A panel is one region inside a
            // view, and this window's is the party.
            panel: "party".to_string(),
            note: None,
        },
        remembered,
    )?;
    let mut screen = interface.screen();
    let mut keys = interface.keys(&game, hello.save_dir.as_deref());

    // A read only happens once the socket says it has something, so this
    // timeout is a backstop against a half-written line rather than the
    // normal wait.
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|e| format!("listening to the host: {e}"))?;

    let outcome = draw_and_listen(
        &mut lines,
        &mut stream
            .try_clone()
            .map_err(|e| format!("talking to the host: {e}"))?,
        &interface,
        &mut screen,
        &mut keys,
    );
    let size = interface.size();
    drop(interface);
    outcome.map(|()| size)
}

/// Draws what the host sends, and sends back what the user does.
///
/// The host going is how a run ends, so it is not an error. The user quit the
/// game, or quit gbs, and either way the window has nothing left to show.
fn draw_and_listen(
    lines: &mut BufReader<UnixStream>,
    up: &mut UnixStream,
    interface: &Hud,
    screen: &mut dyn Screen,
    keys: &mut dyn Keys,
) -> Result<(), String> {
    let mut line = String::new();
    loop {
        let (typed, sent) = wait_for_either(lines, Duration::from_millis(100));

        // Every tick, not only the ticks something arrived on. A window being
        // dragged does not make its keyboard readable, so nothing else here
        // notices the drag, and a HUD that reflows only when the next party
        // lands looks like a hang for as long as the poll interval.
        interface.pump_resizes()?;

        if typed {
            // Zero, because the waiting was done above. This reads the event
            // that is already there, and handles a resize as well as a key.
            match keys.wait(Duration::ZERO)? {
                Interrupt::Quit => {
                    tell(up, &quit_message())?;
                    return Ok(());
                }
                Interrupt::Repick { slot, names } => tell(up, &repick_message(slot, &names))?,
                // A key that only changes what is drawn. The host has no
                // business knowing the highlight moved.
                Interrupt::None => {}
            }
        }

        if !sent {
            continue;
        }
        line.clear();
        match lines.read_line(&mut line) {
            Ok(0) => return Ok(()),
            Ok(_) => {}
            Err(e) if would_block(&e) => continue,
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
    }
}

/// Waits until the user types or the host speaks, whichever comes first.
///
/// Both at once, because a view that waited on one would answer the other
/// late: keys would lag a poll behind, or the party would.
fn wait_for_either(lines: &BufReader<UnixStream>, slice: Duration) -> (bool, bool) {
    // Bytes already buffered will never make the socket readable again, and a
    // whole message can be sitting in there. Polling first would stall the
    // window until the host next said something.
    if !lines.buffer().is_empty() {
        return (false, true);
    }
    let stdin = std::io::stdin();
    let mut fds = [
        nix::poll::PollFd::new(stdin.as_fd(), nix::poll::PollFlags::POLLIN),
        nix::poll::PollFd::new(lines.get_ref().as_fd(), nix::poll::PollFlags::POLLIN),
    ];
    let ms = u16::try_from(slice.as_millis()).unwrap_or(u16::MAX);
    match nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(ms)) {
        Ok(0) => (false, false),
        Ok(_) => {
            let ready = |fd: &nix::poll::PollFd| fd.revents().is_some_and(|r| !r.is_empty());
            (ready(&fds[0]), ready(&fds[1]))
        }
        // A signal is not worth closing a window over. Try both.
        Err(_) => (true, true),
    }
}

/// The user asked to stop. Ending the run is the host's job: it owns the
/// emulator handle, and 011 forbids a window taking the game down with it.
pub fn quit_message() -> serde_json::Value {
    serde_json::json!({ "kind": "quit" })
}

/// The user picked a different save slot, and the view already asked the
/// question. What crosses the socket is the answer, so there is still one
/// wizard rather than two that can disagree.
pub fn repick_message(slot: char, names: &[String]) -> serde_json::Value {
    serde_json::json!({ "kind": "repick", "slot": slot.to_string(), "names": names })
}

/// Sends one message up to the host.
///
/// A host that has gone is not an error. The commonest way to see it is the
/// user quitting the game while the slot question is up in this window: the
/// view is blocked on its own keyboard, misses the end of the socket, and
/// finds out here. The run is over either way, so the window closes quietly
/// rather than with a broken pipe on screen.
pub fn tell(stream: &mut UnixStream, message: &serde_json::Value) -> Result<(), String> {
    let outcome = stream
        .write_all(message.to_string().as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
        .and_then(|()| stream.flush());
    match outcome {
        Ok(()) => Ok(()),
        Err(e) if host_has_gone(&e) => Ok(()),
        Err(e) => Err(format!("telling the host: {e}")),
    }
}

/// Whether this failure means the run ended rather than something broke.
fn host_has_gone(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::NotConnected
    )
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

//! The host: the one process that reads the emulator, and hands the party out.
//!
//! There is one host per run, and it is the process that launched DOSBox.
//! That is not an arrangement of convenience. Yama permits a memory read of a
//! descendant, so only DOSBox's parent may read it, and only one process can
//! be that. Every window is therefore a view: a separate process that draws
//! what it is sent and never reads anything. ADR 0005 has the whole argument.
//!
//! The host is the two seams of the watch loop wearing a socket. It is a
//! [`Screen`], which writes the party out, and a [`Keys`], which spends the
//! pause listening to the views. The loop does not know either end moved.
//! They share one [`Inner`] through an `Rc<RefCell<_>>` for that reason and
//! for no other, the same as the HUD does.

use std::cell::RefCell;
use std::io::{ErrorKind, Read, Write};
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use serde_json::json;

use squire_core::session::Party;

use crate::watch::{Interrupt, Keys, Screen};
use crate::wire;

/// The hello is a message, so it is defined on the wire. It is named here too,
/// because building one is a thing the host's caller does.
pub use crate::wire::Hello;

/// One connected view.
struct Client {
    stream: UnixStream,
    /// Bytes read but not yet a whole line. A socket hands over whatever
    /// arrived, which is not always a message.
    partial: Vec<u8>,
}

struct Inner {
    listener: UnixListener,
    path: PathBuf,
    clients: Vec<Client>,
    hello: String,
    /// The last party and the last notice, resent to a view that arrives
    /// late. This is the state, not a transcript: a view opened an hour in
    /// wants what is true now, and replaying an hour of polls would be a
    /// flicker of history nobody asked for.
    last_party: Option<String>,
    last_notice: Option<String>,
    /// The host's own terminal, which is the log.
    log: Box<dyn Write>,
}

impl Inner {
    /// Sends one line to every view, dropping the ones that have gone.
    ///
    /// A view closing is normal and is not worth a word: views are throwaway,
    /// and the run is the host.
    fn broadcast(&mut self, line: &str) {
        self.clients.retain_mut(|client| {
            client
                .stream
                .write_all(line.as_bytes())
                .and_then(|()| client.stream.write_all(b"\n"))
                .and_then(|()| client.stream.flush())
                .is_ok()
        });
    }

    /// Takes every view waiting to connect, and catches each one up.
    fn accept_waiting(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }
                    let mut client = Client {
                        stream,
                        partial: Vec::new(),
                    };
                    let mut lines = vec![self.hello.clone()];
                    lines.extend(self.last_party.clone());
                    lines.extend(self.last_notice.clone());
                    let welcomed = lines.iter().all(|line| {
                        client
                            .stream
                            .write_all(line.as_bytes())
                            .and_then(|()| client.stream.write_all(b"\n"))
                            .and_then(|()| client.stream.flush())
                            .is_ok()
                    });
                    if welcomed {
                        self.clients.push(client);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                // A failed accept is one view that did not open. The run is
                // not over because a window would not start.
                Err(_) => return,
            }
        }
    }

    /// Reads what view `index` has sent, and returns the first decision in it.
    ///
    /// `closed` says the view went away, which is not an interrupt.
    fn read_client(&mut self, index: usize) -> (Option<Interrupt>, bool) {
        let mut buf = [0u8; 4096];
        let mut found = None;
        let mut closed = false;
        loop {
            match self.clients[index].stream.read(&mut buf) {
                Ok(0) => {
                    closed = true;
                    break;
                }
                Ok(n) => self.clients[index].partial.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => {
                    closed = true;
                    break;
                }
            }
        }

        while let Some(end) = self.clients[index].partial.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.clients[index].partial.drain(..=end).collect();
            if found.is_some() {
                continue;
            }
            if let Some(interrupt) = decode(&line[..line.len() - 1]) {
                found = Some(interrupt);
            }
        }
        (found, closed)
    }
}

/// One line from a view. Anything unreadable is ignored.
///
/// A view is another program. Ending a sitting because one sent nonsense
/// would be the tool taking the game down with it, which 011 forbids.
fn decode(line: &[u8]) -> Option<Interrupt> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    match value["kind"].as_str()? {
        "quit" => Some(Interrupt::Quit),
        "repick" => {
            let slot = value["slot"].as_str()?.chars().next()?;
            let names = value["names"]
                .as_array()?
                .iter()
                .map(|n| n.as_str().map(str::to_string))
                .collect::<Option<Vec<String>>>()?;
            Some(Interrupt::Repick { slot, names })
        }
        _ => None,
    }
}

/// The socket goes when the run does. A stale socket file is a view that
/// connects to nothing and waits forever.
impl Drop for Inner {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// The host, held by the caller for as long as the watch runs.
pub struct Host {
    inner: Rc<RefCell<Inner>>,
}

impl Host {
    /// Starts listening. Nothing has to be connected, then or ever.
    ///
    /// `log` is where the notices are written for a person to read, which in
    /// a real run is the terminal the user typed in.
    pub fn start(path: &Path, hello: Hello, log: Box<dyn Write>) -> Result<Host, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("making {}: {e}", dir.display()))?;
        }
        // A socket file left by a host that was killed. Binding over it fails,
        // and nothing is listening on it, so it is rubbish rather than a
        // running program.
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)
            .map_err(|e| format!("listening on {}: {e}", path.display()))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("setting up {}: {e}", path.display()))?;

        Ok(Host {
            inner: Rc::new(RefCell::new(Inner {
                listener,
                path: path.to_path_buf(),
                clients: Vec::new(),
                hello: hello.line(),
                last_party: None,
                last_notice: None,
                log,
            })),
        })
    }

    /// The drawing seam of the watch loop.
    pub fn screen(&self) -> HostScreen {
        HostScreen {
            inner: Rc::clone(&self.inner),
        }
    }

    /// The pause and the keyboard seam of the watch loop.
    pub fn keys(&self) -> HostKeys {
        HostKeys {
            inner: Rc::clone(&self.inner),
        }
    }

    /// Where the host is listening.
    pub fn path(&self) -> PathBuf {
        self.inner.borrow().path.clone()
    }
}

/// Where the party goes: out to every view, and the notices also to the log.
pub struct HostScreen {
    inner: Rc<RefCell<Inner>>,
}

impl Screen for HostScreen {
    fn party(&mut self, party: &Party) {
        let line = json!({ "kind": "party", "party": wire::party_value(party) }).to_string();
        let mut inner = self.inner.borrow_mut();
        inner.last_party = Some(line.clone());
        inner.broadcast(&line);
    }

    fn notice(&mut self, message: &str) {
        let line = json!({ "kind": "notice", "message": message }).to_string();
        let mut inner = self.inner.borrow_mut();
        inner.last_notice = Some(line.clone());
        inner.broadcast(&line);
        // The same words a view shows also belong in the log, because the log
        // is the only record of a run whose window was closed.
        let _ = writeln!(inner.log, "gbs: {message}");
        let _ = inner.log.flush();
    }
}

/// The pause between polls, spent listening to the views.
pub struct HostKeys {
    inner: Rc<RefCell<Inner>>,
}

impl Keys for HostKeys {
    fn wait(&mut self, pause: Duration) -> Result<Interrupt, String> {
        let deadline = Instant::now() + pause;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let mut inner = self.inner.borrow_mut();

            // The fds are borrowed from `inner`, so they are gathered, polled
            // and dropped before anything is read: the reading needs `inner`
            // mutably and a borrowed fd would still be holding it.
            let readable = {
                let listener = inner.listener.as_fd();
                let mut fds = vec![nix::poll::PollFd::new(
                    listener,
                    nix::poll::PollFlags::POLLIN,
                )];
                for client in &inner.clients {
                    fds.push(nix::poll::PollFd::new(
                        client.stream.as_fd(),
                        nix::poll::PollFlags::POLLIN,
                    ));
                }
                let ms = u16::try_from(left.as_millis()).unwrap_or(u16::MAX);
                match nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(ms)) {
                    Ok(0) => return Ok(Interrupt::None),
                    Ok(_) => (0..fds.len())
                        .filter(|i| fds[*i].revents().is_some_and(|r| !r.is_empty()))
                        .collect::<Vec<usize>>(),
                    // A signal is not worth ending a run over. Spend what is
                    // left of the pause and poll again.
                    Err(_) => {
                        drop(inner);
                        std::thread::sleep(left);
                        return Ok(Interrupt::None);
                    }
                }
            };

            if readable.contains(&0) {
                inner.accept_waiting();
            }
            // Highest first, so that removing one does not renumber the rest.
            let mut found = None;
            for slot in readable.into_iter().filter(|i| *i > 0).rev() {
                let index = slot - 1;
                if index >= inner.clients.len() {
                    continue;
                }
                let (interrupt, closed) = inner.read_client(index);
                if closed {
                    inner.clients.remove(index);
                }
                if found.is_none() {
                    found = interrupt;
                }
            }
            drop(inner);
            if let Some(interrupt) = found {
                return Ok(interrupt);
            }
            if Instant::now() >= deadline {
                return Ok(Interrupt::None);
            }
        }
    }
}

/// Where this run's socket goes: one per run, under the runtime directory.
///
/// Per run rather than per user, so that two games at once is not a special
/// case. The runtime directory is cleared at logout, so a host that was
/// killed leaves nothing behind for the next one to trip over.
pub fn socket_path(runtime_dir: Option<&Path>, pid: u32) -> PathBuf {
    let base = runtime_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    base.join("goldbox-squire").join(format!("{pid}.sock"))
}

/// This run's socket, from the environment.
pub fn default_socket_path() -> PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    socket_path(runtime.as_deref(), std::process::id())
}

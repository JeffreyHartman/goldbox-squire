//! Starts the emulator as a child process.
//!
//! This is the whole permission model. Reading another process's memory needs
//! `PTRACE_MODE_ATTACH`, which Yama restricts. At the common setting,
//! `kernel.yama.ptrace_scope = 1`, a process may attach to its own descendants
//! and to nothing else. Starting DOSBox from Goldbox Squire therefore makes the
//! read permitted with no privilege change and no system-wide setting.
//!
//! The rejected alternatives are worth naming. `setcap cap_sys_ptrace+ep` grants
//! the right to read every process on the machine, which is far more than this
//! tool needs. Telling the user to run `sysctl kernel.yama.ptrace_scope=0`
//! weakens their whole machine, and it resets on every reboot.

use std::ffi::OsStr;
use std::ffi::OsString;
use std::process::{Child, Command, Stdio};

use crate::mem::ProcessReader;
use crate::Error;

/// How to start the emulator.
#[derive(Debug, Clone)]
pub struct Emulator {
    program: OsString,
    args: Vec<OsString>,
}

impl Emulator {
    /// Names the emulator binary. It is looked up on `PATH` unless the name is
    /// a path.
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Emulator {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
        }
    }

    /// Adds one argument.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn program(&self) -> &OsStr {
        &self.program
    }

    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    /// Starts the emulator and returns a handle to it.
    pub fn start(self) -> Result<Launched, Error> {
        let child = Command::new(&self.program)
            .args(&self.args)
            // The emulator keeps the terminal, so its own messages reach the
            // user unchanged.
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| Error::CannotStart {
                program: self.program.to_string_lossy().into_owned(),
                source,
            })?;

        Ok(Launched { child })
    }
}

/// A running emulator started by this process.
///
/// Dropping this handle deliberately does **not** stop the emulator. A tool that
/// reads the game must never take the game down with it.
#[derive(Debug)]
pub struct Launched {
    child: Child,
}

impl Launched {
    pub fn pid(&self) -> i32 {
        self.child.id() as i32
    }

    /// A reader for this emulator's memory.
    pub fn reader(&self) -> ProcessReader {
        ProcessReader::new(self.pid())
    }

    /// Whether the emulator is still running.
    ///
    /// This reaps the child when it ended, so the process does not linger as a
    /// zombie while the tool keeps polling.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Stops the emulator and waits for it to end.
    ///
    /// Stopping an emulator that already ended is not an error.
    pub fn stop(&mut self) -> Result<(), Error> {
        if let Ok(Some(_)) = self.child.try_wait() {
            return Ok(());
        }
        match self.child.kill() {
            Ok(()) => {}
            // The process ended between the check and the signal.
            Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => return Ok(()),
            Err(source) => {
                return Err(Error::Io {
                    pid: self.pid(),
                    source,
                })
            }
        }
        let _ = self.child.wait();
        Ok(())
    }
}

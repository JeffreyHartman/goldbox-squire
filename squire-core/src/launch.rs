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
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::mem::ProcessReader;
use crate::Error;

/// How to start the emulator.
#[derive(Debug, Clone)]
pub struct Emulator {
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    log: Option<PathBuf>,
}

impl Emulator {
    /// Names the emulator binary. It is looked up on `PATH` unless the name is
    /// a path.
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Emulator {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            current_dir: None,
            log: None,
        }
    }

    /// Adds one argument.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    /// Adds one configuration file. Call once per file, in launch order:
    /// later files override earlier ones.
    pub fn conf(self, path: impl AsRef<OsStr>) -> Self {
        self.arg("-conf").arg(path)
    }

    /// Adds one DOS command, run at startup after the confs' autoexec
    /// sections. Call once per command, in order.
    pub fn command(self, cmd: impl AsRef<OsStr>) -> Self {
        self.arg("-c").arg(cmd)
    }

    /// Sets the folder the emulator starts in. A manual install's conf can
    /// use relative mounts, so that path is the folder holding the conf.
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    /// Sends the child's stdout and stderr to this file.
    ///
    /// Without it they are inherited, which lets the emulator print over the
    /// party table. The log keeps a failed launch diagnosable; name its path
    /// in the launch message so the messages stay reachable.
    pub fn log_to(mut self, path: impl AsRef<Path>) -> Self {
        self.log = Some(path.as_ref().to_owned());
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
        let cannot_start = |source| Error::CannotStart {
            program: self.program.to_string_lossy().into_owned(),
            source,
        };

        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }
        // The child never gets the terminal's stdin: gbs owns the keyboard,
        // the emulator gets its own window.
        command.stdin(Stdio::null());
        match &self.log {
            Some(path) => {
                let log = std::fs::File::create(path).map_err(cannot_start)?;
                let log2 = log.try_clone().map_err(cannot_start)?;
                command.stdout(log).stderr(log2);
            }
            // Without a log the output stays on the terminal, which suits a
            // caller that runs no interface of its own.
            None => {
                command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            }
        }

        let child = command.spawn().map_err(cannot_start)?;
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

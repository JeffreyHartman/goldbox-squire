//! Goldbox Squire: reads live party state out of a running DOSBox process.
//!
//! The crate knows nothing about a terminal, a window, or a compositor. A front
//! end drives it and decides how to draw what it returns.

pub mod launch;
pub mod maps;
pub mod mem;
pub mod record;
pub mod saves;
pub mod scan;
pub mod session;
pub mod table;

/// Everything that can go wrong in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("character record table: {0}")]
    Table(String),

    #[error("a record is {want} bytes, but only {got} were given")]
    ShortRecord { want: usize, got: usize },

    #[error("not a character record: {0}")]
    NotARecord(String),

    #[error(
        "cannot read the memory of process {pid}: permission denied.\n\
         Goldbox Squire normally starts DOSBox itself, which makes this work \
         without changing any system setting. Reading a DOSBox that was started \
         separately needs kernel.yama.ptrace_scope to be 0, which weakens the \
         whole machine. Start the game through `gbs` instead."
    )]
    PermissionDenied { pid: i32 },

    #[error("process {pid} is not running")]
    NoSuchProcess { pid: i32 },

    #[error("address {addr:#x} is not mapped in process {pid}")]
    Unmapped { pid: i32, addr: usize },

    #[error("game folder: {0}")]
    GameFolder(String),

    #[error("cannot start `{program}`: {source}")]
    CannotStart {
        program: String,
        #[source]
        source: std::io::Error,
    },

    #[error("reading process {pid}: {source}")]
    Io {
        pid: i32,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    /// Turns an errno from a read into the error a user can act on.
    ///
    /// The exact values matter. Yama reports a blocked read as `EPERM` from
    /// `process_vm_readv` and as `EACCES` from opening `/proc/<pid>/mem`.
    pub(crate) fn from_errno(pid: i32, addr: usize, e: nix::errno::Errno) -> Error {
        use nix::errno::Errno;
        match e {
            Errno::EPERM | Errno::EACCES => Error::PermissionDenied { pid },
            Errno::ESRCH => Error::NoSuchProcess { pid },
            Errno::EFAULT | Errno::EIO => Error::Unmapped { pid, addr },
            other => Error::Io {
                pid,
                source: std::io::Error::from_raw_os_error(other as i32),
            },
        }
    }

    pub(crate) fn from_io_reading(pid: i32, e: std::io::Error) -> Error {
        match e.raw_os_error() {
            Some(libc::EACCES) | Some(libc::EPERM) => Error::PermissionDenied { pid },
            Some(libc::ESRCH) | Some(libc::ENOENT) => Error::NoSuchProcess { pid },
            _ => Error::Io { pid, source: e },
        }
    }
}

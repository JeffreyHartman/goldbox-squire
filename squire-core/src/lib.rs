//! Goldbox Squire: reads live party state out of a running DOSBox process.
//!
//! The crate knows nothing about a terminal, a window, or a compositor. A front
//! end drives it and decides how to draw what it returns.

pub mod maps;
pub mod record;
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
}

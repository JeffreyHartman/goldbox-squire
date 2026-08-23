//! Shared test helpers.
//!
//! Each integration test is its own binary and includes this file, so a
//! helper one of them does not use is not dead code.
#![allow(dead_code)]

use std::path::PathBuf;

/// The six real character records, committed under `tests/fixtures`.
///
/// This panics when a fixture is missing. A test that quietly skips reports a
/// pass while checking nothing, which is worse than a failure.
pub fn saves() -> Vec<Vec<u8>> {
    let dir = fixture_dir();
    (1..=6)
        .map(|i| {
            let path = dir.join(format!("CHRDATA{i}.SAV"));
            std::fs::read(&path)
                .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()))
        })
        .collect()
}

/// The Pool of Radiance record table, via the registry.
pub fn table() -> squire_core::table::Table {
    squire_core::games::find("pool-of-radiance")
        .expect("Pool of Radiance is compiled in")
        .table
}

/// The party's names, in marching order.
pub fn names() -> Vec<String> {
    let table = table();
    saves()
        .iter()
        .map(|s| squire_core::record::decode(&table, s).unwrap().name)
        .collect()
}

pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

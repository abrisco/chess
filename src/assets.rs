//! Utilities for interacting with assets.

use std::path::PathBuf;

/// Convert the given path to a relative repo path.
#[cfg(feature = "cargo")]
pub fn asset_path(path: &str) -> PathBuf {
    // TODO: Figure out a better strategy for locating assets.
    if let Some(stripped) = path.strip_prefix("assets/") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(path)
    }
}

#[cfg(not(feature = "cargo"))]
lazy_static::lazy_static! {
    static ref RUNFILES: runfiles::Runfiles = {
        runfiles::Runfiles::create().expect("No runfiles directory locatable at startup.")
    };
}

/// Return the runfiles path for a given asset.
#[cfg(not(feature = "cargo"))]
pub fn asset_path(path: &str) -> PathBuf {
    let runfile_path = format!("_main/{}", path);
    runfiles::rlocation!(RUNFILES, runfile_path)
        .unwrap_or_else(|e| panic!("Failed to locate runfile: {}\n{:?}", path, e))
}

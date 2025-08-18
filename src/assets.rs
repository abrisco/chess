//! Utilities for interacting with assets.

use std::path::PathBuf;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

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

/// A resource for maintaining a collection of assets loaded into the game.
#[derive(Resource, Default)]
pub struct AssetLibrary(HashMap<String, Handle<Gltf>>);

impl AssetLibrary {
    pub fn get(&self, id: &String) -> Option<&Handle<Gltf>> {
        self.0.get(id)
    }

    pub fn insert(&mut self, id: String, asset: Handle<Gltf>) {
        if self.0.contains_key(&id) {
            panic!("Double inserted asset: {}", id);
        }

        self.0.insert(id, asset);
    }

    pub fn is_all_assets_loaded(&self, asset_server: &Res<AssetServer>) -> bool {
        for mesh in self.0.values() {
            if !asset_server.is_loaded_with_dependencies(mesh) {
                return false;
            }
        }

        true
    }
}

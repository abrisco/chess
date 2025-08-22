//! Utilities for interacting with assets.

use std::path::PathBuf;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

#[cfg(not(feature = "cargo"))]
lazy_static::lazy_static! {
    static ref RUNFILES: runfiles::Runfiles = {
        runfiles::Runfiles::create().expect("No runfiles directory locatable at startup.")
    };
}

#[cfg(feature = "cargo")]
fn loader_path() -> PathBuf {
    PathBuf::from("./")
}

#[cfg(not(feature = "cargo"))]
fn loader_path() -> PathBuf {
    let path = runfiles::find_runfiles_dir().expect("A runfiles directory is required.");

    // When running via `bazel run`, try to locate the execroot.
    let manifest = path.join("MANIFEST");
    if std::env::var_os("BUILD_WORKSPACE_DIRECTORY").is_some() && manifest.exists() {
        let mut current = path.clone();
        loop {
            if let Some(file_name) = current.file_name() {
                if file_name == "execroot" && current.is_dir() {
                    return current;
                }
            }

            // move to parent
            if !current.pop() {
                break;
            }
        }
    }

    path
}

lazy_static::lazy_static! {
    pub static ref LOADER_PATH: PathBuf = {
        loader_path()
    };
}

/// Convert the given path to a relative repo path.
#[cfg(feature = "cargo")]
pub fn asset_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

/// Return the runfiles path for a given asset.
#[cfg(not(feature = "cargo"))]
pub fn asset_path(path: &str) -> PathBuf {
    let runfile_path = format!("_main/{}", path);
    let path = runfiles::rlocation!(RUNFILES, runfile_path)
        .unwrap_or_else(|| panic!("Failed to locate runfile: {}", path));

    // Attempt to make paths relative.
    path.strip_prefix(LOADER_PATH.as_path())
        .expect("Assets should always be relative to the loader path.")
        .to_path_buf()
}

/// A resource for maintaining a collection of assets loaded into the game.
#[derive(Resource, Default)]
pub struct AssetLibrary {
    pub scenes: HashMap<String, Handle<Gltf>>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
}

impl AssetLibrary {
    pub fn get_scene(&self, id: &String) -> Option<&Handle<Gltf>> {
        self.scenes.get(id)
    }

    pub fn insert_scene(&mut self, id: String, asset: Handle<Gltf>) {
        if self.scenes.contains_key(&id) {
            panic!("Double inserted asset: {}", id);
        }

        self.scenes.insert(id, asset);
    }

    pub fn get_material(&self, id: &String) -> Option<&Handle<StandardMaterial>> {
        self.materials.get(id)
    }

    pub fn insert_material(&mut self, id: String, asset: Handle<StandardMaterial>) {
        if self.materials.contains_key(&id) {
            panic!("Double inserted asset: {}", id);
        }

        self.materials.insert(id, asset);
    }

    pub fn is_all_assets_loaded(&self, asset_server: &Res<AssetServer>) -> bool {
        for mesh in self.scenes.values() {
            if !asset_server.is_loaded_with_dependencies(mesh) {
                return false;
            }
        }

        for mesh in self.materials.values() {
            if !asset_server.is_loaded_with_dependencies(mesh) {
                return false;
            }
        }

        true
    }
}

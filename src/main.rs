
mod components;
mod plugins;

use bevy::DefaultPlugins;
use bevy::prelude::{App, Update, Startup};
use plugins::HelloPlugin;

fn main() {
    App::new()
    .add_plugins((DefaultPlugins, HelloPlugin))
    .run();
}


mod components;
mod plugins;

use bevy::DefaultPlugins;
use bevy::prelude::{App, Update, Startup};
use plugins::{HelloPlugin, GamePlugin};

fn main() {
    App::new()
    .add_plugins((DefaultPlugins, GamePlugin))
    .run();
}


#[cfg(test)]
mod test {
    #[test]
    fn do_a_thing() {
        let mut foo = 1;
        if foo > 0 {
            foo += 1;
        }
        assert_eq!(foo, 2);
    }
}

use bevy::prelude::{App, Plugin, Commands, Query, With, Startup, Update};

use crate::components::{Person, Name};

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, Self::add_people)
        .add_systems(Update, Self::greet_people);
    }
}

impl HelloPlugin {
    fn add_people(mut commands: Commands) {
        commands.spawn((Person, Name("Elaina Proctor".to_string())));
        commands.spawn((Person, Name("Renzo Hume".to_string())));
        commands.spawn((Person, Name("Zayna Nieves".to_string())));
    }
    
    fn greet_people(query: Query<&Name, With<Person>>) {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

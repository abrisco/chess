use bevy::prelude::{
    default, shape, App, Camera3dBundle, Color, Commands, Mesh, PbrBundle, Plugin, Query, Startup,
    Transform, Update, Vec3, With, ResMut, Assets, StandardMaterial, PointLightBundle, PointLight,
};

use crate::components::{Name, Person};

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::add_people)
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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::startup);
    }
}

impl GamePlugin {
    fn startup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0., 1.5, 6.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });

        // Plane
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        });

        // Light
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        });
    }
}

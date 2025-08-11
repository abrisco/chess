use bevy::log::debug;
use bevy::prelude::*;

use crate::AppState;

#[derive(Clone)]
enum Action {
    Exit,
    Play,
}

impl From<&Action> for AppState {
    fn from(value: &Action) -> Self {
        match value {
            Action::Exit => AppState::Shutdown,
            Action::Play => AppState::GameLoading,
        }
    }
}

#[derive(Component)]
struct MenuButton(Action);

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

#[derive(Resource)]
pub struct Menu(Entity);

impl Menu {
    fn on_loading(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
        let root = commands
            .spawn((
                Node {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Button,
                        MenuButton(Action::Play),
                        Node {
                            width: Val::Px(150.),
                            height: Val::Px(65.),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Play"),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        )],
                    ),
                    (
                        Button,
                        MenuButton(Action::Exit),
                        Node {
                            width: Val::Px(150.),
                            height: Val::Px(65.),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Exit"),
                            TextFont {
                                font_size: 33.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        )],
                    )
                ],
            ))
            .id();

        // Track the menu items
        commands.insert_resource(Menu(root));

        // Spawn the camera so the UI widgets can be seen.
        commands.spawn(Camera2d);

        // TODO: Wait for any assets to be loaded before transitioning.
        next_state.set(AppState::Menu);
    }

    /// Handle button presses and perform their associated actions.
    fn on_update(
        mut next_state: ResMut<NextState<AppState>>,
        mut interaction_query: Query<
            (&Interaction, &MenuButton),
            (Changed<Interaction>, With<MenuButton>),
        >,
    ) {
        for (interaction, action) in &mut interaction_query {
            match interaction {
                Interaction::Pressed => {
                    let new_state = AppState::from(&action.0);
                    debug!("NEW STATE: {:?}", new_state);
                    next_state.set(new_state);
                }
                _ => {}
            }
        }
    }

    /// Cleanup any menu items.
    fn on_exit(
        mut commands: Commands,
        menu: Res<Menu>,
        camera_query: Query<Entity, With<Camera2d>>,
    ) {
        commands.entity(menu.0).despawn();

        for camera in camera_query.iter() {
            commands.entity(camera).despawn();
        }
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MenuLoading), Menu::on_loading)
            .add_systems(Update, Menu::on_update.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), Menu::on_exit);
    }
}

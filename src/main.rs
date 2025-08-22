use bevy::prelude::*;

mod chess;
use chess::*;

mod menu;
use menu::*;

mod assets;

#[derive(Debug, Default, States, Hash, PartialEq, Eq, Clone)]
pub enum AppState {
    #[default]
    Startup,
    MenuLoading,
    Menu,
    GameLoading,
    Game,
    Shutdown,
}

fn on_startup(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::GameLoading);
}

fn on_shutdown(mut exit: EventWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn main() {
    let loader_path = assets::LOADER_PATH.to_string_lossy().to_string();
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: loader_path.clone(),
            processed_file_path: loader_path,
            ..Default::default()
        }))
        .init_state::<AppState>()
        .add_systems(Startup, on_startup)
        .add_systems(OnEnter(AppState::Shutdown), on_shutdown)
        .add_plugins(MenuPlugin)
        .add_plugins(ChessPlugin)
        .run();
}

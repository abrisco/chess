use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::assets::asset_path;
use crate::AppState;

mod board_coords;
use board_coords::*;

#[derive(
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
)]
pub enum ChessPieceType {
    PAWN,
    ROOK,
    KNIGHT,
    BISHOP,
    QUEEN,
    KING,
}

#[derive(Component)]
pub struct ChessPiece(ChessPieceType);

#[derive(Resource)]
pub struct ChessPieceMeshes(HashMap<ChessPieceType, Handle<Scene>>);

impl ChessPiece {
    pub fn on_loading(commands: &mut Commands, asset_server: &Res<AssetServer>) {
        let mut pieces: HashMap<ChessPieceType, Handle<Scene>> = HashMap::new();

        for piece in ChessPieceType::iter() {
            let path = match piece {
                ChessPieceType::PAWN => asset_path("assets/models/pawn.glb"),
                ChessPieceType::ROOK => asset_path("assets/models/rook.glb"),
                ChessPieceType::KNIGHT => asset_path("assets/models/knight.glb"),
                ChessPieceType::BISHOP => asset_path("assets/models/bishop.glb"),
                ChessPieceType::QUEEN => asset_path("assets/models/queen.glb"),
                ChessPieceType::KING => asset_path("assets/models/king.glb"),
            };
            assert!(
                pieces
                    .insert(
                        piece.clone(),
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))
                    )
                    .is_none(),
                "Double initialized ChessPiece asset: {}",
                piece
            );
        }

        commands.insert_resource(ChessPieceMeshes(pieces));
    }

    pub fn spawn(kind: ChessPieceType) {
        unimplemented!()
    }
}

#[derive(Default, Clone)]
pub struct GridCell {
    // The entity currently occupying the cell.
    pub occupant: Option<Entity>,

    // The translation in local space to the root of the cell on the board.
    pub translation: Vec3,
}

impl GridCell {
    pub fn is_occupied(self) -> bool {
        self.occupant.is_some()
    }
}

#[derive(Component)]
pub struct ChessBoard {
    grid: Vec<Vec<GridCell>>,
}

#[derive(Component)]
pub struct ChessBoardGrid(Vec<Vec<GridCell>>);

#[derive(Resource)]
pub struct ChessBoardMesh(Handle<Scene>);

impl ChessBoard {
    pub fn on_loading(commands: &mut Commands, asset_server: &Res<AssetServer>) {
        let handle = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(asset_path("assets/models/chess_board.glb")));

        commands.insert_resource(ChessBoardMesh(handle));
    }

    pub fn spawn(commands: &mut Commands) {
        // Load the board mesh as a resource.

        // Board
        // commands.spawn((
        //     ChessBoard {
        //         grid: vec![vec![GridCell::default(); 8]; 8],
        //     },
        //     SceneRoot(scene),
        // ));
    }

    pub fn get_cell<'this>(&'this self, cell: BoardCoordinates) -> &'this GridCell {
        let (x, y) = cell.as_coords();
        &self.grid[x][y]
    }
}

const CAMERA_START_POSITION: Vec3 = Vec3 {
    x: 0.5,
    y: 0.5,
    z: 0.5,
};

impl ChessPlugin {
    fn on_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 9.0),
        ));

        // Load the board

        // Load all pieces
    }

    fn update_camera(
        mut camera: Single<&mut Transform, With<Camera>>,
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        mouse_motion: Res<AccumulatedMouseMotion>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
        // Placeholder constant focus
        let focus = Vec3::ZERO;

        // Reset position when "R" is pressed
        if keyboard_input.pressed(KeyCode::KeyR) {
            camera.translation = CAMERA_START_POSITION;
            camera.look_at(focus, Vec3::Y);
        }

        // Constants
        let sensitivity = 0.03;

        // If no mouse buttons are pressed, don't update the camera.
        let delta = if mouse_buttons.pressed(MouseButton::Left) {
            mouse_motion.delta * 0.01
        } else if keyboard_input.any_pressed([
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]) {
            Vec2::new(
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    sensitivity
                } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    -sensitivity
                } else {
                    0.0
                },
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    sensitivity
                } else if keyboard_input.pressed(KeyCode::ArrowDown) {
                    -sensitivity
                } else {
                    0.0
                },
            )
        } else {
            return;
        };

        // Current offset from focus
        let offset = camera.translation - focus;

        let up = Vec3::Y;
        let right = camera.rotation * Vec3::X;

        // Compute pitch and yaw
        let pitch = Quat::from_axis_angle(right, -delta.y);
        let yaw = Quat::from_axis_angle(up, -delta.x);
        let rotation = pitch * yaw;

        // Update camera position
        camera.translation = focus + (rotation * offset);

        // Compute the camera's up vector while accounting for any yaw (left/right) movement.
        let camera_up = yaw * camera.rotation * Vec3::Y;

        // Correct the lookat position.
        camera.look_at(focus, camera_up);
    }
}

pub struct ChessPlugin;

impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameLoading), Self::on_loading)
            .add_systems(Update, Self::update_camera.run_if(in_state(AppState::Game)));
    }
}

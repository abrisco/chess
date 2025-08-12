use bevy::gltf::GltfNode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::assets::asset_path;
use crate::AppState;

mod board_coords;
use board_coords::BoardCoordinate as Coord;

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
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Component, strum_macros::Display, strum_macros::EnumIter)]
pub enum Team {
    Black,
    White,
}

#[derive(Component)]
pub struct ChessPiece(ChessPieceType);

impl ChessPiece {
    /// Begin loading resources for chess pieces.
    pub fn on_enter_loading(asset_server: &Res<AssetServer>, asset_library: &mut AssetLibrary) {
        for piece in ChessPieceType::iter() {
            let path = match piece {
                ChessPieceType::Pawn => asset_path("assets/models/pawn.glb"),
                ChessPieceType::Rook => asset_path("assets/models/rook.glb"),
                ChessPieceType::Knight => asset_path("assets/models/knight.glb"),
                ChessPieceType::Bishop => asset_path("assets/models/bishop.glb"),
                ChessPieceType::Queen => asset_path("assets/models/queen.glb"),
                ChessPieceType::King => asset_path("assets/models/king.glb"),
            };

            asset_library.insert(piece.to_string(), asset_server.load(path));
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_library: &Res<AssetLibrary>,
        gltf_assets: &Res<Assets<Gltf>>,
        board: Entity,
        team: Team,
        position: Coord,
        kind: ChessPieceType,
    ) {
        let asset_id = kind.to_string();

        // Load the asset handle
        let handle = asset_library
            .get(&asset_id)
            .expect("Failed to locate BOARD asset.");

        let gltf = gltf_assets
            .get(handle)
            .expect("Failed to locate BOARD Gltf asset.");

        // Get the gltf scene root
        let scene = gltf.default_scene.clone().unwrap();

        // Locate the transform
        let translation = Vec3::new(0.0, 0.0, 0.0);
        let transform = Transform::from_xyz(translation.x, translation.y, translation.z);

        // Spawn in world
        commands.entity(board).with_children(|board| {
            board.spawn((ChessPiece(kind), team, transform, SceneRoot(scene)));
        });
    }
}

#[derive(Default, Clone)]
struct GridCell {
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
    /// Begin loading the chess board resources.
    pub fn on_enter_loading(asset_server: &Res<AssetServer>, asset_library: &mut AssetLibrary) {
        let handle = asset_server.load(asset_path("assets/models/chess_board.glb"));

        asset_library.insert("BOARD".to_string(), handle);
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_library: &Res<AssetLibrary>,
        gltf_assets: &Res<Assets<Gltf>>,
        gltf_node_assets: &Res<Assets<GltfNode>>,
    ) -> Entity {
        let asset_id = "BOARD".to_string();
        // Locate the board resource
        let handle = asset_library
            .get(&asset_id)
            .expect("Failed to locate BOARD asset.");

        let gltf = gltf_assets
            .get(handle)
            .expect("Failed to locate BOARD Gltf asset.");

        let mut grid = vec![vec![GridCell::default(); 8]; 8];
        for position in Coord::iter() {
            let (x, y) = position.as_coords();
            let handle = &gltf.named_nodes[position.into()];
            let node = gltf_node_assets.get(handle).unwrap();
            grid[x][y].translation = node.transform.translation;
        }

        commands
            .spawn((
                ChessBoard { grid },
                Transform::from_xyz(0.0, 0.0, 0.0),
                SceneRoot(gltf.default_scene.as_ref().unwrap().clone()),
            ))
            .id()
    }

    pub fn get_cell<'this>(&'this self, cell: Coord) -> &'this GridCell {
        let (x, y) = cell.as_coords();
        &self.grid[x][y]
    }
}

const CAMERA_START_POSITION: Vec3 = Vec3 {
    x: 0.3,
    y: 0.3,
    z: 0.3,
};

struct Chess;

impl Chess {
    fn on_enter_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Spawn Camera
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(
                CAMERA_START_POSITION.x,
                CAMERA_START_POSITION.y,
                CAMERA_START_POSITION.z,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
        ));

        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 9.0),
        ));

        let mut asset_library = AssetLibrary::default();

        // Chess Board
        ChessBoard::on_enter_loading(&asset_server, &mut asset_library);

        // Chess Pierces
        ChessPiece::on_enter_loading(&asset_server, &mut asset_library);

        commands.insert_resource(asset_library)
    }

    /// Check to see if all known assets have finished loading and we're ready to play the game
    fn on_loading(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        asset_library: Res<AssetLibrary>,
        gltf_assets: Res<Assets<Gltf>>,
        gltf_node_assets: Res<Assets<GltfNode>>,
        mut next_state: ResMut<NextState<AppState>>,
    ) {
        if !asset_library.is_all_assets_loaded(&asset_server) {
            return;
        }

        // Spawn entities in the world.
        let board = ChessBoard::spawn(
            &mut commands,
            &asset_library,
            &gltf_assets,
            &gltf_node_assets,
        );

        // Spawn white team
        #[rustfmt::skip]
        let _ = {
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::A2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::B2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::C2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::D2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::E2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::F2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::G2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::H2, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::A1, ChessPieceType::Rook);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::B1, ChessPieceType::Knight);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::C1, ChessPieceType::Bishop);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::D1, ChessPieceType::Queen);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::E1, ChessPieceType::King);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::F1, ChessPieceType::Bishop);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::G1, ChessPieceType::Knight);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::White, Coord::H1, ChessPieceType::Rook);
        };

        // Spawn black team.
        #[rustfmt::skip]
        let _ = {
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::A7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::B7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::C7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::D7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::E7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::F7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::G7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::H7, ChessPieceType::Pawn);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::A8, ChessPieceType::Rook);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::B8, ChessPieceType::Knight);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::C8, ChessPieceType::Bishop);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::D8, ChessPieceType::Queen);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::E8, ChessPieceType::King);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::F8, ChessPieceType::Bishop);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::G8, ChessPieceType::Knight);
            ChessPiece::spawn(&mut commands, &asset_library, &gltf_assets, board, Team::Black, Coord::H8, ChessPieceType::Rook);
        };

        // Trigger the next state.
        next_state.set(AppState::Game)
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
        app.add_systems(OnEnter(AppState::GameLoading), Chess::on_enter_loading)
            .add_systems(
                Update,
                (Chess::on_loading,).run_if(in_state(AppState::GameLoading)),
            )
            .add_systems(
                Update,
                (Chess::update_camera,).run_if(in_state(AppState::Game)),
            );
    }
}

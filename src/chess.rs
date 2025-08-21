use bevy::gltf::GltfNode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use strum::IntoEnumIterator;

use crate::assets::{asset_path, AssetLibrary};
use crate::AppState;

mod board_coords;
use board_coords::BoardCoordinate as Coord;

#[derive(
    Debug,
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

#[derive(Debug, Component, Clone, Copy, strum_macros::EnumIter, Eq, PartialEq)]
pub enum Team {
    Black,
    White,
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Black => write!(f, "Team::Black"),
            Team::White => write!(f, "Team::White"),
        }
    }
}

#[derive(Resource)]
pub struct ActiveTeam(Team);

impl Default for ActiveTeam {
    fn default() -> Self {
        Self(Team::White)
    }
}

#[derive(Debug, Event)]
pub struct PieceMoveEvent {
    pub board: Entity,
    pub from: Coord,
    pub to: Coord,
}

#[derive(Resource, Default)]
pub struct PieceSelection {
    pub piece: Option<Entity>,
}

impl PieceSelection {
    pub fn observer_select_piece(
        mut trigger: Trigger<Pointer<Pressed>>,
        pieces_query: Query<(&mut ChessPiece, &ChildOf)>,
        board_query: Query<&ChessBoard>,
        active_team: Res<ActiveTeam>,
        mut selection: ResMut<PieceSelection>,
        mut writer: EventWriter<PieceMoveEvent>,
    ) {
        trigger.propagate(false);
        let piece_entity = trigger.target();
        let (piece, piece_relationship) = pieces_query.get(piece_entity).unwrap();

        match selection.piece {
            Some(selected) => {
                println!("Selected({:?} New({:?})", selected, piece_entity);
                if selected == piece_entity {
                    // Unselect the piece if it was selected twice.
                    selection.piece = None;
                    println!("Piece unselected! {:?}", piece_entity);
                } else {
                    let board_entity = piece_relationship.parent();
                    let board = board_query.get(board_entity).unwrap();
                    let piece_pos = board.occupants[&piece_entity];
                    let selected_pos = board.occupants[&selected];

                    // Unselect if the same square was chosen
                    if piece_pos == selected_pos {
                        selection.piece = None;
                        println!("Piece unselected! {:?}", piece_entity);
                    } else {
                        // Check if another piece was selected and determine what move
                        // that could translate to.
                        let movement = PieceMoveEvent {
                            board: board_entity,
                            from: selected_pos,
                            to: piece_pos,
                        };
                        println!("Submitting movement: {:?}", movement);
                        writer.write(movement);
                        selection.piece = None;
                    }
                }
            }
            None => {
                // Select the piece only if it represents the active team's piece
                if piece.team == active_team.0 {
                    println!("Piece selected! {:?}", piece_entity);
                    selection.piece = Some(piece_entity);
                } else {
                    println!("Not the active team: {}", active_team.0)
                }
            }
        }
    }

    fn observer_select_square(
        mut trigger: Trigger<Pointer<Pressed>>,
        board_query: Query<(&ChessBoard, &Transform)>,
        piece_query: Query<&ChessPiece>,
        active_team: Res<ActiveTeam>,
        mut selection: ResMut<PieceSelection>,
        mut writer: EventWriter<PieceMoveEvent>,
    ) {
        trigger.propagate(false);

        // Locate the board
        let board_entity = trigger.target();
        let (board, board_transform) = board_query.get(board_entity).unwrap();

        // In world space (for some reason).
        let trigger_pos = trigger.event().hit.position.unwrap();

        // Find the closest point to the target
        let coord = {
            let mut distances = Coord::iter()
                .map(|coord| {
                    let cell_pos = board.get_cell_translation(&coord, &board_transform);
                    (trigger_pos.distance(cell_pos), coord)
                })
                .collect::<Vec<(_, _)>>();

            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            distances.first().unwrap().1
        };

        match selection.piece {
            // Submit the move request
            Some(piece) => {
                let movement = PieceMoveEvent {
                    board: board_entity,
                    from: board.occupants[&piece],
                    to: coord,
                };
                println!("Submitting movement: {:?}", movement);
                writer.write(movement);
                selection.piece = None;
            }
            // If selection is empty and the cell is occupied, select the occupant
            None => {
                if let Some(occupant) = board.get_cell(&coord).occupant {
                    let piece = piece_query.get(occupant).unwrap();
                    if piece.team == active_team.0 {
                        selection.piece = Some(occupant);
                        println!("Selected occupant at {}", coord);
                    } else {
                        println!("Not active team at {}", coord)
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct PieceNeedsTeamMaterial(Team);

#[derive(Debug, Component)]
pub struct ChessPiece {
    kind: ChessPieceType,
    team: Team,

    /// Track if the piece has moved. Useful for castling or a pawns double steps
    has_moved: bool,
}

impl ChessPiece {
    /// Begin loading resources for chess pieces.
    pub fn on_enter_loading(
        asset_server: &Res<AssetServer>,
        asset_library: &mut AssetLibrary,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for piece in ChessPieceType::iter() {
            let path = match piece {
                ChessPieceType::Pawn => asset_path("assets/models/pawn.glb"),
                ChessPieceType::Rook => asset_path("assets/models/rook.glb"),
                ChessPieceType::Knight => asset_path("assets/models/knight.glb"),
                ChessPieceType::Bishop => asset_path("assets/models/bishop.glb"),
                ChessPieceType::Queen => asset_path("assets/models/queen.glb"),
                ChessPieceType::King => asset_path("assets/models/king.glb"),
            };

            asset_library.insert_scene(piece.to_string(), asset_server.load(path));
        }

        // asset_library.insert_material(Team::White.to_string(), materials.add(Color::WHITE));
        // asset_library.insert_material(Team::Black.to_string(), materials.add(Color::BLACK));
    }

    /// Adds the correct color to pieces after they've spawned.
    fn on_spawn_scene(
        mut commands: Commands,
        query: Query<(Entity, &SceneInstance, &PieceNeedsTeamMaterial), Added<SceneInstance>>,
        asset_library: Res<AssetLibrary>,
        scene_spawner: ResMut<SceneSpawner>,
    ) {
        for (root, instance, PieceNeedsTeamMaterial(team)) in &query {
            let mat = asset_library
                .get_material(&team.to_string())
                .unwrap_or_else(|| panic!("Unexpected material requested: {}", team));

            // Iterate over entities spawned from the scene
            for entity in scene_spawner.iter_instance_entities(**instance) {
                commands.entity(entity).insert(MeshMaterial3d(mat.clone()));
            }

            // Cleanup: we don’t need the marker anymore
            commands.entity(root).remove::<PieceNeedsTeamMaterial>();
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_library: &Res<AssetLibrary>,
        gltf_assets: &Res<Assets<Gltf>>,
        board: &mut ChessBoard,
        board_transform: &Transform,
        team: Team,
        position: Coord,
        kind: ChessPieceType,
    ) -> Entity {
        let asset_id = kind.to_string();

        // Load the asset handle
        let handle = asset_library
            .get_scene(&asset_id)
            .expect("Failed to locate BOARD asset.");

        let gltf = gltf_assets
            .get(handle)
            .expect("Failed to locate BOARD Gltf asset.");

        // Get the gltf scene root
        let scene = gltf.default_scene.clone().unwrap();

        // Locate the transform
        let transform = board.get_cell_transform(&position, &board_transform, &team);

        // Spawn in world
        let entity = commands
            .spawn((
                ChessPiece {
                    kind: kind.clone(),
                    team,
                    has_moved: false,
                },
                team,
                transform,
                SceneRoot(scene),
            ))
            .observe(PieceSelection::observer_select_piece)
            .id();

        board.insert_piece(entity, position);

        entity
    }
}

#[derive(Debug, Default, Clone)]
struct GridCell {
    // The entity currently occupying the cell.
    pub occupant: Option<Entity>,

    // The translation in local space to the root of the cell on the board.
    translation: Vec3,
}

#[derive(Debug, Component)]
pub struct ChessBoard {
    grid: Vec<Vec<GridCell>>,
    occupants: HashMap<Entity, Coord>,
}

impl ChessBoard {
    /// Begin loading the chess board resources.
    pub fn on_enter_loading(asset_server: &Res<AssetServer>, asset_library: &mut AssetLibrary) {
        let handle = asset_server.load(asset_path("assets/models/chess_board.glb"));

        asset_library.insert_scene("BOARD".to_string(), handle);
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_library: &Res<AssetLibrary>,
        gltf_assets: &Res<Assets<Gltf>>,
        gltf_node_assets: &Res<Assets<GltfNode>>,
        with_pieces: bool,
    ) {
        let asset_id = "BOARD".to_string();
        // Locate the board resource
        let handle = asset_library
            .get_scene(&asset_id)
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

        let mut board = ChessBoard {
            grid,
            occupants: HashMap::new(),
        };
        let board_transform = Transform::from_xyz(0.0, 0.0, 0.0);

        let mut pieces = Vec::new();
        if with_pieces {
            #[rustfmt::skip]
            pieces.extend([

                // Spawn white team
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::A2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::B2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::C2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::D2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::E2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::F2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::G2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::H2, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::A1, ChessPieceType::Rook),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::B1, ChessPieceType::Knight),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::C1, ChessPieceType::Bishop),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::D1, ChessPieceType::Queen),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::E1, ChessPieceType::King),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::F1, ChessPieceType::Bishop),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::G1, ChessPieceType::Knight),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::White, Coord::H1, ChessPieceType::Rook),

                // Spawn black team.
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::A7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::B7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::C7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::D7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::E7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::F7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::G7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::H7, ChessPieceType::Pawn),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::A8, ChessPieceType::Rook),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::B8, ChessPieceType::Knight),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::C8, ChessPieceType::Bishop),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::D8, ChessPieceType::Queen),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::E8, ChessPieceType::King),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::F8, ChessPieceType::Bishop),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::G8, ChessPieceType::Knight),
                ChessPiece::spawn(commands, &asset_library, &gltf_assets, &mut board, &board_transform, Team::Black, Coord::H8, ChessPieceType::Rook),
            ]);
        }

        let board_entity = commands
            .spawn((
                board,
                board_transform,
                SceneRoot(gltf.default_scene.as_ref().unwrap().clone()),
            ))
            .observe(PieceSelection::observer_select_square)
            .id();

        for piece in pieces {
            commands.entity(board_entity).add_child(piece);
        }
    }

    fn get_cell<'this>(&'this self, cell: &Coord) -> &'this GridCell {
        let (x, y) = cell.as_coords();
        &self.grid[x][y]
    }

    fn get_cell_mut<'this>(&'this mut self, cell: &Coord) -> &'this mut GridCell {
        let (x, y) = cell.as_coords();
        &mut self.grid[x][y]
    }

    pub fn get_cell_translation(&self, cell: &Coord, board_transform: &Transform) -> Vec3 {
        let cell = self.get_cell(cell);

        board_transform.translation + (cell.translation * board_transform.scale)
    }

    pub fn get_cell_transform(
        &self,
        cell: &Coord,
        board_transform: &Transform,
        team: &Team,
    ) -> Transform {
        let forward = board_transform.rotation
            * (match team {
                Team::Black => Vec3::NEG_Z,
                Team::White => Vec3::Z,
            })
            .normalize();

        let rotation = Quat::from_rotation_arc(Vec3::Z, forward);

        Transform::from_translation(self.get_cell_translation(cell, board_transform))
            .with_rotation(rotation)
    }

    pub fn insert_piece(&mut self, piece: Entity, position: Coord) {
        // Update the board grid to occupy the specified position.
        let cell = self.get_cell_mut(&position);

        assert!(cell.occupant.is_none(), "Cell {} is occupied.", position,);
        cell.occupant = Some(piece);
        self.occupants.insert(piece, position);
    }

    pub fn move_piece(
        &mut self,
        piece_to_move: Entity,
        piece_transform: &mut Transform,
        team: &Team,
        to: Coord,
        board_transform: &Transform,
    ) {
        let from_position = self
            .occupants
            .insert(piece_to_move, to)
            .unwrap_or_else(|| panic!("Entity is not an occupant of the board."));

        // Update the destination
        {
            let to_cell = self.get_cell_mut(&to);
            assert!(
                to_cell.occupant.is_none(),
                "Cell {} has an occupant that needs to be cleared first.",
                &to
            );
            to_cell.occupant = Some(piece_to_move);
        }

        // Clear the previous cell
        {
            let from_cell = self.get_cell_mut(&from_position);
            from_cell.occupant = None;
        }

        *piece_transform = self.get_cell_transform(&to, board_transform, team);
    }

    pub fn remove_piece(&mut self, piece: Entity) {
        let position = self
            .occupants
            .remove(&piece)
            .unwrap_or_else(|| panic!("Entity is not an occupant of the board."));

        let cell = self.get_cell_mut(&position);
        cell.occupant = None;
    }
}

const CAMERA_START_POSITION: Vec3 = Vec3 {
    x: 0.3,
    y: 0.3,
    z: 0.3,
};

const CAMERA_FOCUS: Vec3 = Vec3::ZERO;

struct Chess;

impl Chess {
    fn on_enter_loading(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        materials: ResMut<Assets<StandardMaterial>>,
    ) {
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
        ChessPiece::on_enter_loading(&asset_server, &mut asset_library, materials);

        // Allocate any necessary resources.
        commands.insert_resource(asset_library);
        commands.insert_resource(ActiveTeam(Team::White));
        commands.insert_resource(PieceSelection::default())
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
        // Wait for all assets to be fully loaded.
        if !asset_library.is_all_assets_loaded(&asset_server) {
            return;
        }

        // Spawn board and all pieces
        ChessBoard::spawn(
            &mut commands,
            &asset_library,
            &gltf_assets,
            &gltf_node_assets,
            true,
        );

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
        let focus = CAMERA_FOCUS;

        // Reset position when "R" is pressed
        if keyboard_input.pressed(KeyCode::KeyR) {
            camera.translation = CAMERA_START_POSITION;
            camera.look_at(focus, Vec3::Y);
        }

        // Constants
        let sensitivity = 0.03;

        // If no mouse buttons are pressed, don't update the camera.
        let delta = if mouse_buttons.pressed(MouseButton::Middle) {
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

    fn update_move(
        mut move_events: EventReader<PieceMoveEvent>,
        mut commands: Commands,
        mut pieces_query: Query<(&ChessPiece, &mut Transform), Without<ChessBoard>>,
        mut boards_query: Query<(&mut ChessBoard, &Transform), Without<ChessPiece>>,
        mut active_team: ResMut<ActiveTeam>,
    ) {
        for event in move_events.read() {
            println!("Handling move: {:?}", event);
            let (mut board, board_transform) = match boards_query.get_mut(event.board) {
                Ok(b) => b,
                Err(e) => panic!("Unable to find chess board: {:?}", e),
            };

            let from_cell = board.get_cell_mut(&event.from);
            let from_occupant = match from_cell.occupant {
                Some(o) => o,
                None => panic!(
                    "A move was made for a cell that has no occupant: {:?}",
                    event
                ),
            };

            // Deleting any pieces that were taken
            let to_cell = board.get_cell(&event.to);
            if let Some(to_occupant) = to_cell.occupant {
                board.remove_piece(to_occupant);
                commands.entity(to_occupant).despawn();
            }

            // Move piece from one square to another
            let (from_piece, mut from_transform) = pieces_query
                .get_mut(from_occupant)
                .expect("Failed to get from piece");

            board.move_piece(
                from_occupant,
                &mut from_transform,
                &from_piece.team,
                event.to,
                &board_transform,
            );

            // Replacing any pawns that were upgraded.
            if from_piece.kind == ChessPieceType::Pawn {
                // TODO:
            }

            // Toggle the active team
            active_team.0 = match active_team.0 {
                Team::Black => Team::White,
                Team::White => Team::Black,
            };
        }
    }
}
pub struct ChessPlugin;

impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_event::<PieceMoveEvent>()
            .add_systems(OnEnter(AppState::GameLoading), Chess::on_enter_loading)
            .add_systems(
                Update,
                Chess::on_loading.run_if(in_state(AppState::GameLoading)),
            )
            .add_systems(
                Update,
                (
                    Chess::update_camera.run_if(in_state(AppState::Game)),
                    Chess::update_move.run_if(in_state(AppState::Game)),
                    ChessPiece::on_spawn_scene,
                ),
            );
    }
}

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}

const CAMERA_START_X: f32 = 0.5;
const CAMERA_START_Y: f32 = 0.5;
const CAMERA_START_Z: f32 = 0.5;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::startup)
            .add_systems(Update, Self::update_camera);
    }
}

impl GamePlugin {
    fn startup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        // circular base plane
        // commands.spawn((
        //     Mesh3d(meshes.add(Circle::new(4.0))),
        //     MeshMaterial3d(materials.add(Color::WHITE)),
        //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        // ));

        // Board
        commands.spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/chess_board.glb")),
        ));

        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 9.0),
        ));

        // camera
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(CAMERA_START_X, CAMERA_START_Y, CAMERA_START_Z)
                .looking_at(Vec3::ZERO, Vec3::Y),
        ));
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
            camera.translation = Vec3::new(CAMERA_START_X, CAMERA_START_Y, CAMERA_START_Z);
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
        let right = Vec3::X;

        // Compute pitch and yaw
        let pitch = Quat::from_axis_angle(right, -delta.y);
        let yaw = Quat::from_axis_angle(up, -delta.x);
        let rotation = pitch * yaw;

        // Update camera position
        camera.translation = focus + (rotation * offset);

        camera.look_at(focus, up);
    }
}

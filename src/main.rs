use std::f32::consts::FRAC_PI_2;
use std::ops::Range;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::math::VectorSpace;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}

#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    // Clamp pitch to this range
    pub pitch_range: Range<f32>,
    pub roll_speed: f32,
    pub yaw_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            // These values are completely arbitrary, chosen because they seem to produce
            // "sensible" results for this example. Adjust as required.
            orbit_distance: 20.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            roll_speed: 1.0,
            yaw_speed: 0.004,
        }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::startup)
            .add_systems(Update, Self::update_camera)
            .init_resource::<CameraSettings>();
    }
}

impl GamePlugin {
    fn startup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // circular base
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
        // cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
        // camera
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }

    fn update_camera(
        mut camera: Single<&mut Transform, With<Camera>>,
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        mouse_motion: Res<AccumulatedMouseMotion>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
        // Reset position when "R" is pressed
        if keyboard_input.pressed(KeyCode::KeyR) {
            camera.translation = Vec3::new(-2.5, 4.5, 9.0);
            camera.look_at(Vec3::ZERO, Vec3::Y);
        }

        // If no mouse buttons are pressed, don't update the camera.
        let delta = if mouse_buttons.pressed(MouseButton::Left) {
            mouse_motion.delta
        } else if keyboard_input.any_pressed([
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]) {
            Vec2::new(
                if keyboard_input.pressed(KeyCode::ArrowRight) {
                    3.0
                } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
                    -3.0
                } else {
                    0.0
                },
                if keyboard_input.pressed(KeyCode::ArrowUp) {
                    3.0
                } else if keyboard_input.pressed(KeyCode::ArrowDown) {
                    -3.0
                } else {
                    0.0
                },
            )
        } else {
            return;
        };

        // Constants
        let sensitivity = 0.01;

        // Placeholder constant focus
        let focus = Vec3::ZERO;

        // Current offset from focus
        let offset = camera.translation - focus;

        // Compute the new rotation based on mouse input
        let forward = -offset.normalize(); // viewing direction is toward focus

        // Use the camera's current local up to avoid instability
        let local_up = camera.rotation * Vec3::Y;

        // If forward is too close to up, fall back to another axis. This occurs because
        // as two vectors approach being identical their cross product nears zero.
        let pole_threshold = 0.999;
        let fallback_up = camera.rotation * Vec3::Z;
        let view_up = if forward.dot(local_up).abs() > pole_threshold {
            fallback_up
        } else {
            local_up
        };

        let right = forward.cross(view_up).normalize();
        let up = right.cross(forward).normalize();

        // Compute pitch and yaw
        let pitch = Quat::from_axis_angle(right, -delta.y * sensitivity);
        let yaw = Quat::from_axis_angle(up, -delta.x * sensitivity);
        let rotation = yaw * pitch;

        // Update camera position
        camera.translation = focus + (rotation * offset);

        // Update look_at using up as it's a safe reference point.
        camera.look_at(focus, up);
    }
}

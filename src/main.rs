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
        camera_settings: Res<CameraSettings>,
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        mouse_motion: Res<AccumulatedMouseMotion>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
    ) {
        if keyboard_input.pressed(KeyCode::KeyR) {
            camera.translation = Vec3::new(-2.5, 4.5, 9.0);
            camera.look_at(Vec3::ZERO, Vec3::Y);
        }

        if !mouse_buttons.pressed(MouseButton::Left) {
            return
        }

        // Build the rotation quaternion
        let delta = mouse_motion.delta;
        println!("{:?}", delta);
        let scale = 0.01;
        let yaw   = Quat::from_axis_angle(Vec3::X, -delta.y * scale);   // Up/Down
        let pitch = Quat::from_axis_angle(Vec3::Y, -delta.x * scale);  // Left/Right
        let rotation = yaw * pitch; // Apply pitch first, then yaw

        // Placeholder constant focus
        let focus = Vec3::ZERO;

        // Compute offset for rotation
        let offset = camera.translation - focus;

        let position = rotation * offset;
        println!("POSITION: {:?}", position);

        let up = rotation * camera.up();
        camera.translation = position;
        // camera.rotation = rotation;
        camera.look_at(focus, up);


        // Do Rotation

        // let camera_translation = camera.translation.clone();
        // let camera_rotation = camera.rotation.clone();

        // let delta = mouse_motion.delta;
        // let mut delta_roll = 0.0;

        // if mouse_buttons.pressed(MouseButton::Left) {
        //     delta_roll -= 1.0;
        // }
        // if mouse_buttons.pressed(MouseButton::Right) {
        //     delta_roll += 1.0;
        // }

        // // Mouse motion is one of the few inputs that should not be multiplied by delta time,
        // // as we are already receiving the full movement since the last frame was rendered. Multiplying
        // // by delta time here would make the movement slower that it should be.
        // let delta_pitch = delta.y * camera_settings.pitch_speed;
        // let delta_yaw = delta.x * camera_settings.yaw_speed;

        // // // Conversely, we DO need to factor in delta time for mouse button inputs.
        // // delta_roll *= camera_settings.roll_speed * time.delta_secs();

        // // Obtain the existing pitch, yaw, and roll values from the transform.
        // let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

        // // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
        // let pitch = (pitch + delta_pitch).clamp(
        //     camera_settings.pitch_range.start,
        //     camera_settings.pitch_range.end,
        // );
        // let roll = roll + delta_roll;
        // let yaw = yaw + delta_yaw;
        // Quat::from
        // camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

        // // Adjust the translation to maintain the correct orientation toward the orbit target.
        // // In our example it's a static target, but this could easily be customized.
        // let target = Vec3::ZERO;
        // camera.translation = target - camera.forward() * camera_settings.orbit_distance;
    }
}

use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{EventReader, EventWriter, KeyCode, Query, Res, ResMut, Time, Transform};
use bevy_window::{CursorGrabMode, Window};
use random_word::Lang;
use spacetimedb_sdk::DbContext;
use crate::module_bindings::{set_name, set_position, spawn_entity, DbTransform, DbVector3, DbVector4, PlayerTableAccess};
use crate::plugins::environment::systems::camera_system::CameraController;
use crate::plugins::network::systems::database::DbConnectionResource;

/// Example system to input a camera using double-precision for position.
pub fn flight_systems(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>, /*
                                               mouse_button_input: Res<ButtonInput<MouseButton>>,*/
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut windows: Query<&mut Window>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
    mut app_exit_events: EventWriter<AppExit>,
    //mut ctx: ResMut<DbConnectionResource>,
) {
    
    let mut window = windows.single_mut();
    let (mut transform, mut controller) = query.single_mut();

    // ====================
    // 1) Handle Mouse Look
    // ====================
    if !window.cursor_options.visible {
        for event in mouse_motion_events.read() {
            // Adjust yaw/pitch in f32
            controller.yaw -= event.delta.x * controller.sensitivity;
            controller.pitch += event.delta.y * controller.sensitivity;
            controller.pitch = controller.pitch.clamp(-89.9, 89.9);

            // Convert degrees to radians (f32)
            let yaw_radians = controller.yaw.to_radians();
            let pitch_radians = controller.pitch.to_radians();

            // Build a double-precision quaternion from those angles
            let rot_yaw = Quat::from_axis_angle(Vec3::Y, yaw_radians);
            let rot_pitch = Quat::from_axis_angle(Vec3::X, -pitch_radians);

            transform.rotation = rot_yaw * rot_pitch;
        }
    }

    // ====================
    // 2) Adjust Movement Speed with Mouse Wheel
    // ====================
    for event in mouse_wheel_events.read() {
        let base_factor = 1.1_f32;
        let factor = base_factor.powf(event.y);
        controller.speed *= factor;
        if controller.speed < 0.01 {
            controller.speed = 0.01;
        }
    }


    // ====================
    // 3) Handle Keyboard Movement (WASD, Space, Shift)
    // ====================
    let mut direction = Vec3::ZERO;

    // Forward/Back
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= transform.forward().as_vec3();
    }

    // Left/Right
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= transform.right().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }

    // Up/Down
    if keyboard_input.pressed(KeyCode::Space) {
        direction += transform.up().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        direction -= transform.up().as_vec3();
    }

    // Normalize direction if needed
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    // Apply movement in double-precision
    let delta_seconds = time.delta_secs_f64();
    let distance = controller.speed as f64 * delta_seconds;
    transform.translation += direction * distance as f32;

    /*ctx.0.reducers.set_position(DbVector3{
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
    }).expect("TODO: panic message");
*/



}
use crate::helper::egui_dock::MainCamera;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_render::camera::{Exposure, PhysicalCameraParameters, Projection};
use bevy_window::CursorGrabMode;
use rand::Rng;
use random_word::Lang;
use crate::module_bindings::{set_name, set_position, spawn_entity, DbTransform, DbVector3, DbVector4};
use crate::plugins::network::systems::database::DbConnectionResource;

#[derive(Component)]
pub struct CameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,
            sensitivity: 0.1,
        }
    }
}

pub fn setup(mut commands: Commands,
             db_resource: Res<DbConnectionResource>,) {
    

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 10.0), // initial f32
        GlobalTransform::default(),
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            near: 0.0001,
            ..default()
        }),
        MainCamera,
        CameraController::default(),
        Exposure::from_physical_camera(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
            sensor_height: 0.01866,
        }),

    ));
}

fn random_vec3(min: f32, max: f32) -> Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(
        rng.gen_range(min..max),
        rng.gen_range(min..max),
        rng.gen_range(min..max),
    )
}

/// Example system to control a camera using double-precision for position.
pub fn camera_controller_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>, /*
                                               mouse_button_input: Res<ButtonInput<MouseButton>>,*/
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut windows: Query<&mut Window>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut ctx: ResMut<DbConnectionResource>,
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

    let word = random_word::get(Lang::En);

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        ctx.0.reducers.set_name(word.to_string()).unwrap();
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let rand_position = random_vec3(-10.0,10.0);
        let rand_rotation = random_vec3(-10.0,10.0);
        let rand_scale = random_vec3(0.1,1.0);
        ctx.0.reducers.spawn_entity(DbTransform{
            position: DbVector3{
                x: rand_position.x,
                y: rand_position.y,
                z: rand_position.z,
            },
            rotation: DbVector4 {
                x: rand_rotation.x,
                y: rand_rotation.y,
                z: rand_position.z,
                w: 0.0,
            },
            scale: DbVector3 {
                x: rand_scale.x,
                y: rand_scale.x,
                z: rand_scale.x,
            },
        }).unwrap();
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

    ctx.0.reducers.set_position(DbVector3{
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
    }).expect("TODO: panic message");
    

    // =========================
    // 4) Lock/Unlock Mouse (L)
    // =========================
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        // Toggle between locked and unlocked
        if window.cursor_options.grab_mode == CursorGrabMode::None {
            // Lock
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        } else {
            // Unlock
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }
    }



    // =======================
    // 7) Exit on Escape
    // =======================
    if keyboard_input.pressed(KeyCode::Escape) {
        app_exit_events.send(Default::default());
    }
}

use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use crate::plugins::environment::systems::camera_system::CameraController;


fn move_by(
    mut q: Query<&mut Transform, With<CameraController>>,
    delta: Vec3,
) {
    for mut t in &mut q {
        t.translation += delta;
    }
}


/// Example system to input a camera using double-precision for position.
pub fn flight_systems(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut windows: Query<&mut Window>,
    // all camera entities carry this tag
    mut xforms: Query<&mut Transform, With<CameraController>>,
    mut ctrls:  Query<&mut CameraController>,
    mut exit_ev: EventWriter<AppExit>,
) {
    //------------------------------------------------------------
    // 0) Early-out if no camera
    //------------------------------------------------------------
    if xforms.is_empty() { return; }

    //------------------------------------------------------------
    // 1) Rotation & speed input (borrow transform/controller)
    //------------------------------------------------------------
    let delta_vec3 = {
        let Ok(mut window)     = windows.get_single_mut() else { return };
        let Ok(mut transform)  = xforms.get_single_mut() else { return };
        let Ok(mut controller) = ctrls.get_single_mut() else { return };

        //------------------ mouse look --------------------------
        if !window.cursor_options.visible {
            for ev in mouse_motion.read() {
                controller.yaw   -= ev.delta.x * controller.sensitivity;
                controller.pitch += ev.delta.y * controller.sensitivity;
                controller.pitch = controller.pitch.clamp(-89.9, 89.9);

                let yaw   = controller.yaw.to_radians();
                let pitch = controller.pitch.to_radians();

                let rot   = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch);
                transform.rotation = rot;
            }
        }

        //------------------ mouse wheel speed -------------------
        for ev in mouse_wheel.read() {
            controller.speed = (controller.speed * 1.1_f32.powf(ev.y)).max(0.01);
        }

        //------------------ keyboard direction -----------------
        let mut dir = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) { dir += *transform.forward(); }
        if keyboard.pressed(KeyCode::KeyS) { dir -= *transform.forward(); }
        if keyboard.pressed(KeyCode::KeyA) { dir -= *transform.right();   }
        if keyboard.pressed(KeyCode::KeyD) { dir += *transform.right();   }
        if keyboard.pressed(KeyCode::Space)       { dir += *transform.up(); }
        if keyboard.pressed(KeyCode::ShiftLeft) ||
            keyboard.pressed(KeyCode::ShiftRight)  { dir -= *transform.up(); }

        if dir.length_squared() > 0.0 { dir = dir.normalize(); }

        //------------------ compute delta ----------------------
        let distance = controller.speed * time.delta_secs_f64() as f32;
        dir * distance
    }; // ⬅ scopes end here; mutable borrows are dropped

    //------------------------------------------------------------
    // 2) Apply translation with the helper
    //------------------------------------------------------------
    move_by(xforms, delta_vec3);

}
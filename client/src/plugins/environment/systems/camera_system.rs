
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

pub fn setup(mut commands: Commands,) {
    

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 10.0), // initial f32
        GlobalTransform::default(),
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            near: 0.0001,
            ..default()
        }),
        CameraController::default(),
        Exposure::from_physical_camera(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
            sensor_height: 0.01866,
        }),

    ));
}


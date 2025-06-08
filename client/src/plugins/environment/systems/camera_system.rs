use bevy::core_pipeline::Skybox;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::camera::{Exposure, PhysicalCameraParameters};
use big_space::prelude::{BigSpaceCommands, FloatingOrigin};
use rand::Rng;
use crate::plugins::big_space::big_space_plugin::RootGrid;

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
             root:           Res<RootGrid>,
             asset_server: Res<AssetServer>) {



    let cubemap_handle = asset_server.load("textures/skybox_space_1024/sky.ktx2");
    commands.insert_resource(PendingSkybox { handle: cubemap_handle.clone() });
    
    commands.entity(root.0).with_children(|parent| {
        parent.spawn((

            Name::new("Camera"),
            Transform::from_xyz(0.0, 0.0, 10.0), // initial position
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
            FloatingOrigin,
            Skybox {
                image: cubemap_handle.clone(),
                brightness: 1000.0,
                ..default()
            },
        ));
    });

}



#[derive(Resource)]
struct PendingSkybox {
    handle: Handle<Image>,
}


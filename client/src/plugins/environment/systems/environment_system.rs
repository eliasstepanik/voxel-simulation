
use bevy::prelude::*;
use big_space::prelude::{BigSpace, BigSpaceCommands, GridCell, GridCommands};
use crate::plugins::big_space::big_space_plugin::RootGrid;

/// Earth and a FIFA-size football (diameters, metres)
const EARTH_DIAM: f32 = 12_742_000.0;   // 12 742 km
const BALL_DIAM:  f32 = 0.22;           // 22 cm

pub(crate) fn setup(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    root:          Res<RootGrid>,
) {
    // one unit-diameter sphere mesh, reused for every instance
    let sphere_mesh = meshes.add(Sphere::new(0.5).mesh().ico(32).unwrap());
    let mat         = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.7, 0.8),
        perceptual_roughness: 0.7,
        ..default()
    });

    // light
    commands.entity(root.0).with_children(|p| {
        p.spawn((
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_4,
                0.0,
                0.0,
            )),
            GlobalTransform::default(),
            DirectionalLight {
                shadows_enabled: true,
                illuminance: 1.0,
                ..default()
            },
        ));
    });

    /*// ---------- spawn spheres from football-size up to Earth-size ----------
    const N: usize = 10;                       // how many spheres
    let log_min = BALL_DIAM.log10();
    let log_max = EARTH_DIAM.log10();

    let mut offset = 0.0_f32;                  // keep objects apart
    commands.entity(root.0).with_children(|parent| {
        for i in 0..N {
            // log-spaced diameters
            let t        = i as f32 / (N as f32 - 1.0);
            let diam     = 10f32.powf(log_min + t * (log_max - log_min));
            let radius   = diam * 0.5;
            let scale_v3 = Vec3::splat(radius);          // unit sphere → real size

            // place the sphere so they don’t overlap
            offset += radius;                            // move by previous radius
            let pos = Vec3::new(offset, radius, 0.0);    // sit on X axis, resting on Y=0
            offset += radius + radius * 0.05;            // add gap (5 %)

            parent.spawn((
                // spatial requirements for big_space
                GridCell::ZERO,
                Transform::from_scale(scale_v3).with_translation(pos),
                GlobalTransform::default(),
                // rendering
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(mat.clone()),
                Name::new(format!("Sphere_{i}")),
            ));
        }
    });*/
}


pub fn update(time: Res<Time>,)  {

}
use bevy::asset::RenderAssetUsages;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;
use bevy::render::mesh::*;
use big_space::floating_origins::FloatingOrigin;
use big_space::prelude::GridCell;
use noise::{Fbm, NoiseFn, Perlin};
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::camera_system::CameraController;


#[derive(Component)]
pub struct PlanetMaker;



#[derive(Resource)]
pub struct PlanetNoise(pub Fbm<Perlin>);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    root: Res<RootGrid>,
) {
    // Diameter ~ Earth (~12,742 km) × 2 to exaggerate terrain if desired
    let radius = 12_742_000.0;
    let sphere_mesh = meshes.add(
        SphereMeshBuilder::new(radius, SphereKind::Ico { subdivisions: 100 })
            .build(),
    );
    let material_handle = materials.add(StandardMaterial::from(Color::srgb(0.3, 0.6, 1.0)));

    commands.entity(root.0).with_children(|parent| {
        parent.spawn((
            Name::new("Planet"),
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material_handle),
            GridCell::ZERO,
            Transform::default(),
            PlanetMaker,
            Wireframe,
        ));
    });
}


pub(crate) fn setup_noise(mut commands: Commands) {
    let fbm_noise = Fbm::<Perlin>::new(0);
    commands.insert_resource(PlanetNoise(fbm_noise));
}

pub fn deform_planet(
    mut meshes: ResMut<Assets<Mesh>>,
    noise: Res<PlanetNoise>,
    query: Query<&Mesh3d, With<PlanetMaker>>,
) {
    let frequency = 4.0 / 12_742_000.0;
    let amplitude = 100_000.0;

    for mesh3d in query.iter() {
        let handle: &Handle<Mesh> = &mesh3d.0;

        if let Some(mesh) = meshes.get_mut(handle) {
            // 1. Immutable borrow to extract normals (or default)
            let normals: Vec<[f32; 3]> = if let Some(VertexAttributeValues::Float32x3(vals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            {
                vals.clone()
            } else {
                // default normals if none exist
                let count = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .and_then(|attr| match attr {
                        VertexAttributeValues::Float32x3(v) => Some(v.len()),
                        _ => None,
                    })
                    .unwrap_or(0);
                vec![[0.0, 1.0, 0.0]; count]
            };

            // 2. Drop the immutable borrow, then mutable-borrow positions
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                // Now mutate positions using the pre-fetched normals
                for (i, pos) in positions.iter_mut().enumerate() {
                    let mut vertex = Vec3::new(pos[0], pos[1], pos[2]);
                    let normal = Vec3::new(
                        normals[i][0],
                        normals[i][1],
                        normals[i][2],
                    );

                    let unit_dir = vertex.normalize();
                    let sample = [
                        unit_dir.x as f64 * frequency as f64,
                        unit_dir.y as f64 * frequency as f64,
                        unit_dir.z as f64 * frequency as f64,
                    ];
                    let noise_value = noise.0.get(sample) as f32;
                    let offset = normal * (noise_value * amplitude);

                    let new_pos = unit_dir * (vertex.length() + offset.length());
                    *pos = [new_pos.x, new_pos.y, new_pos.z];
                }

                
                mesh.compute_smooth_normals();
            }

            // Force AABB recalc
            mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR);
        }
    }
}
use bevy::asset::RenderAssetUsages;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;
use bevy::render::mesh::*;
use big_space::floating_origins::FloatingOrigin;
use big_space::prelude::GridCell;
use noise::{Fbm, NoiseFn, Perlin};
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::camera_system::CameraController;
use crate::plugins::environment::systems::planet_system::PlanetMaker;
use crate::plugins::environment::systems::voxels::structure::*;

pub fn setup(
    mut commands: Commands,
    root: Res<RootGrid>,
) {
    let unit_size = 1.0;

    let octree_base_size = 64.0 * unit_size; // Octree's total size in your world space
    let octree_depth = 10;


    let mut octree = SparseVoxelOctree::new(octree_depth, octree_base_size as f32, false, false, false);


    let color = Color::rgb(0.2, 0.8, 0.2);
    /*generate_voxel_rect(&mut octree,color);*/
    generate_voxel_sphere(&mut octree, 100, color);

    commands.entity(root.0).with_children(|parent| {
        parent.spawn(
            (
                Transform::default(),
                octree
            )
        );
    });
}

fn generate_voxel_sphere(
    octree: &mut SparseVoxelOctree,
    planet_radius: i32,
    voxel_color: Color,
) {
    // For simplicity, we center the sphere around (0,0,0).
    // We'll loop over a cubic region [-planet_radius, +planet_radius] in x, y, z
    let min = -planet_radius;
    let max = planet_radius;

    let step = octree.get_spacing_at_depth(octree.max_depth);

    for ix in min..=max {
        let x = ix;
        for iy in min..=max {
            let y = iy;
            for iz in min..=max {
                let z = iz;

                // Check if within sphere of radius `planet_radius`
                let dist2 = x * x + y * y + z * z;
                if dist2 <= planet_radius * planet_radius {
                    // Convert (x,y,z) to world space, stepping by `voxel_step`.
                    let wx = x as f32 * step;
                    let wy = y as f32 * step;
                    let wz = z as f32 * step;
                    let position = Vec3::new(wx, wy, wz);

                    // Insert the voxel
                    let voxel = Voxel {
                        color: voxel_color,
                    };
                    octree.insert(position, voxel);
                }
            }
        }
    }
}



/// Inserts a 16x256x16 "column" of voxels into the octree at (0,0,0) corner.
/// If you want it offset or centered differently, just adjust the for-loop ranges or offsets.
fn generate_voxel_rect(
    octree: &mut SparseVoxelOctree,
    voxel_color: Color,
) {
    // The dimensions of our rectangle: 16 x 256 x 16
    let size_x = 16;
    let size_y = 256;
    let size_z = 16;

    // We'll get the voxel spacing (size at the deepest level), same as in your sphere code.
    let step = octree.get_spacing_at_depth(octree.max_depth);

    // Triple-nested loop for each voxel in [0..16, 0..256, 0..16]
    for ix in 0..size_x {
        let x = ix as f32;
        for iy in 0..size_y {
            let y = iy as f32;
            for iz in 0..size_z {
                let z = iz as f32;

                // Convert (x,y,z) to world coordinates
                let wx = x * step;
                let wy = y * step;
                let wz = z * step;

                let position = Vec3::new(wx, wy, wz);

                // Insert the voxel
                let voxel = Voxel {
                    color: voxel_color,
                };
                octree.insert(position, voxel);
            }
        }
    }
}

fn generate_large_plane(
    octree: &mut SparseVoxelOctree,
    width: usize,
    depth: usize,
    color: Color,
) {
    // We'll get the voxel spacing (size at the deepest level).
    let step = octree.get_spacing_at_depth(octree.max_depth);

    // Double-nested loop for each voxel in [0..width, 0..depth],
    // with y=0.
    for ix in 0..width {
        let x = ix as f32;
        for iz in 0..depth {
            let z = iz as f32;
            // y is always 0.
            let y = 0.0;

            // Convert (x,0,z) to world coordinates
            let wx = x * step;
            let wy = y * step;
            let wz = z * step;

            let position = Vec3::new(wx, wy, wz);

            // Insert the voxel
            let voxel = Voxel {
                color,
            };
            octree.insert(position, voxel);
        }
    }
}


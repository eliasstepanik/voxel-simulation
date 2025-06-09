use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::structure::*;

use bevy::prelude::*;
use bevy::render::mesh::*;
use noise::{NoiseFn, Perlin};

pub fn setup(
    mut commands: Commands,
    root: Res<RootGrid>,
) {
    let unit_size = 1.0_f32;
    let octree_base_size = 64.0 * unit_size;
    let octree_depth = 10;

    // 1. Create octree and wrap in Arc<Mutex<>> for thread-safe generation
    let mut octree = SparseVoxelOctree::new(octree_depth, octree_base_size, false, false, false);

    // 2. Generate sphere in parallel, dropping the cloned Arc inside the function
    let color = Color::rgb(0.2, 0.8, 0.2);

    generate_voxel_sphere(&mut octree, 50, color);
    /*generate_large_plane(&mut octree, 4000, 4000, color);*/

    // 4. Spawn entity with both Transform and the real octree component
    commands.entity(root.0).with_children(|parent| {
        parent.spawn((Transform::default(), octree));
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


pub fn generate_solid_plane_with_noise(
    octree: &mut SparseVoxelOctree,
    width: usize,
    depth: usize,
    color: Color,
    noise: &Perlin,
    frequency: f32,
    amplitude: f32,
) {
    // Size of one voxel at the deepest level
    let step = octree.get_spacing_at_depth(octree.max_depth);

    for ix in 0..width {
        let x = ix as f32;
        for iz in 0..depth {
            let z = iz as f32;

            // Sample Perlin noise at scaled coordinates
            let sample_x = x * frequency;
            let sample_z = z * frequency;
            let noise_val = noise.get([sample_x as f64, sample_z as f64]) as f32;

            // Height in world units
            let height_world = noise_val * amplitude;
            // Convert height to number of voxel layers
            let max_layer = (height_world / step).ceil() as usize;

            // Fill from layer 0 up to max_layer
            for iy in 0..=max_layer {
                let position = Vec3::new(
                    x * step,
                    iy as f32 * step,
                    z * step,
                );

                let voxel = Voxel { color };
                octree.insert(position, voxel);
            }
        }
    }
}

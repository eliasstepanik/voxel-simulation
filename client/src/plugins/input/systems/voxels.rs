use crate::plugins::environment::systems::camera_system::CameraController;
use crate::plugins::environment::systems::voxels::octree;
use crate::plugins::environment::systems::voxels::structure::*;
use bevy::prelude::*;
use std::path::Path;

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum VoxelEditMode {
    Single,
    Sphere,
}

impl Default for VoxelEditMode {
    fn default() -> Self {
        Self::Single
    }
}

const EDIT_SPHERE_RADIUS: i32 = 8;

///TODO
pub fn voxel_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut octree_query: Query<&mut SparseVoxelOctree>,

    mut query: Query<(&mut Transform, &mut CameraController)>,
    mut windows: Query<&mut Window>,
    mut edit_mode: ResMut<VoxelEditMode>,
) {
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };
    let Ok((mut transform, _)) = query.get_single_mut() else {
        return;
    };

    // =======================
    // 5) Octree Keys
    // =======================
    if keyboard_input.just_pressed(KeyCode::F2) {
        for mut octree in octree_query.iter_mut() {
            octree.show_wireframe = !octree.show_wireframe;
        }
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        for mut octree in octree_query.iter_mut() {
            octree.show_world_grid = !octree.show_world_grid;
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) && window.cursor_options.visible == false {
        for mut octree in octree_query.iter_mut() {
            octree.insert(transform.translation, Voxel::random_sides());
        }
    }
    if keyboard_input.just_pressed(KeyCode::F4) {
        let path = Path::new("octree.bin");
        for octree in octree_query.iter() {
            if let Err(e) = octree.save_to_file(path) {
                error!("failed to save octree: {e}");
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::F5) {
        *edit_mode = match *edit_mode {
            VoxelEditMode::Single => VoxelEditMode::Sphere,
            VoxelEditMode::Sphere => VoxelEditMode::Single,
        };
    }

    // =======================
    // 6) Building
    // =======================

    if (mouse_button_input.just_pressed(MouseButton::Left)
        || mouse_button_input.just_pressed(MouseButton::Right))
        && !window.cursor_options.visible
    {
        // Get the mouse position in normalized device coordinates (-1 to 1)
        if let Some(_) = window.cursor_position() {
            // Set the ray direction to the camera's forward vector
            let ray_origin = transform.translation;
            let ray_direction = transform.forward().normalize();

            let ray = Ray {
                origin: ray_origin,
                direction: ray_direction,
            };

            for mut octree in octree_query.iter_mut() {
                if let Some((hit_x, hit_y, hit_z, depth, normal)) = octree.raycast(&ray) {
                    match *edit_mode {
                        VoxelEditMode::Single => {
                            if mouse_button_input.just_pressed(MouseButton::Right) {
                                let voxel_size = octree.get_spacing_at_depth(depth);
                                let hit_position = Vec3::new(hit_x as f32, hit_y as f32, hit_z as f32);
                                let epsilon = voxel_size * 0.1;
                                let offset_position = hit_position - (normal * Vec3::splat(epsilon));
                                octree.remove(offset_position);
                            } else if mouse_button_input.just_pressed(MouseButton::Left) {
                                let voxel_size = octree.get_spacing_at_depth(depth);
                                let hit_position = Vec3::new(hit_x as f32, hit_y as f32, hit_z as f32);
                                let epsilon = voxel_size * 0.1;
                                let offset_position = hit_position + (normal * Vec3::splat(epsilon));
                                octree.insert(offset_position, Voxel::random_sides());
                            }
                        }
                        VoxelEditMode::Sphere => {
                            if mouse_button_input.just_pressed(MouseButton::Right) {
                                let voxel_size = octree.get_spacing_at_depth(depth);
                                let hit_position = Vec3::new(hit_x as f32, hit_y as f32, hit_z as f32);
                                let epsilon = voxel_size * 0.1;
                                let offset = hit_position - normal * Vec3::splat(epsilon);
                                octree.remove_sphere(offset, EDIT_SPHERE_RADIUS);
                            } else if mouse_button_input.just_pressed(MouseButton::Left) {
                                let voxel_size = octree.get_spacing_at_depth(depth);
                                let hit_position = Vec3::new(hit_x as f32, hit_y as f32, hit_z as f32);
                                let epsilon = voxel_size * 0.1;
                                let offset = hit_position + normal * Vec3::splat(epsilon);
                                octree.insert_sphere(offset, EDIT_SPHERE_RADIUS, Voxel::grass_block());
                            }
                        }
                    }
                }
            }
        }
    }
}

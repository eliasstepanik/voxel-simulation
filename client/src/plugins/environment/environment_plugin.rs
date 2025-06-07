use bevy::app::{App, Plugin, PreStartup, PreUpdate, Startup};
use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::structure::SparseVoxelOctree;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Startup,
            (
                crate::plugins::environment::systems::camera_system::setup,
                crate::plugins::environment::systems::environment_system::setup.after(crate::plugins::environment::systems::camera_system::setup),
                crate::plugins::environment::systems::voxel_system::setup

            ),
        );

        app.add_systems( 
            Update,
            (
                // old: voxels::rendering::render,
                crate::plugins::environment::systems::voxels::render_chunks::rebuild_dirty_chunks,
                crate::plugins::environment::systems::voxels::debug::visualize_octree_system
                    .run_if(should_visualize_octree),
                crate::plugins::environment::systems::voxels::debug::draw_grid
                    .run_if(should_draw_grid),
            )
                .chain(),
        );



    }
}


fn should_visualize_octree(octree_query: Query<&SparseVoxelOctree>,) -> bool {
    octree_query.single().show_wireframe
}

fn should_draw_grid(octree_query: Query<&SparseVoxelOctree>,) -> bool {
    octree_query.single().show_world_grid
}

fn should_visualize_chunks(octree_query: Query<&SparseVoxelOctree>,) -> bool {
    octree_query.single().show_chunks
}
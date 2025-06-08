use bevy::app::{App, Plugin, PreStartup, PreUpdate, Startup};
use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::culling::{
    despawn_distant_chunks,
    replace_near_lod_chunks,
};
use crate::plugins::environment::systems::voxels::debug::{draw_grid, visualize_octree_system};
use crate::plugins::environment::systems::voxels::queue_systems;
use crate::plugins::environment::systems::voxels::queue_systems::{
    enqueue_visible_chunks,
    process_chunk_queue,
    enqueue_lod_chunks,
    process_lod_chunk_queue,
};
use crate::plugins::environment::systems::voxels::render_chunks::rebuild_dirty_chunks;
use crate::plugins::environment::systems::voxels::structure::{
    ChunkBudget,
    ChunkCullingCfg,
    ChunkQueue,
    LodChunkQueue,
    SparseVoxelOctree,
    SpawnedChunks,
};

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

        app.insert_resource(ChunkCullingCfg {
            view_distance_chunks: 10,
            lod_distance_chunks: 20,
            lod_level: 2,
        });
        app.insert_resource(ChunkBudget { per_frame: 20 });
        
        app
            // ------------------------------------------------------------------------
            // resources
            // ------------------------------------------------------------------------
            .init_resource::<ChunkQueue>()
            .init_resource::<LodChunkQueue>()
            .init_resource::<SpawnedChunks>()
            // ------------------------------------------------------------------------
            // frame update
            // ------------------------------------------------------------------------
            .add_systems(
                Update,
                (
                    /* ---------- culling & streaming ---------- */
                    despawn_distant_chunks,
                    replace_near_lod_chunks.after(despawn_distant_chunks),
                    enqueue_visible_chunks.after(replace_near_lod_chunks),         // 2.  find new visible ones
                    process_chunk_queue .after(enqueue_visible_chunks),          // 3.  spawn ≤ budget per frame
                    enqueue_lod_chunks.after(process_chunk_queue),
                    process_lod_chunk_queue.after(enqueue_lod_chunks),
                    rebuild_dirty_chunks .after(process_lod_chunk_queue),          // 4.  (re)mesh dirty chunks
                    /* ---------- optional debug drawing ------- */
                    visualize_octree_system
                        .run_if(should_visualize_octree)
                        .after(rebuild_dirty_chunks),
                    draw_grid
                        .run_if(should_draw_grid)
                        .after(visualize_octree_system),
                )
                    .chain(),     // make the whole tuple execute in this exact order
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
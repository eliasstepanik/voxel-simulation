use bevy::app::{App, Plugin, PreStartup, PreUpdate, Startup};
use bevy::prelude::*;
use bevy_easy_compute::prelude::*;
use crate::plugins::environment::systems::voxels::sphere_compute::{SphereWorker, SphereParams, SphereGenerated, execute_sphere_once, apply_sphere_result};
use crate::plugins::environment::systems::voxels::debug::{draw_grid, visualize_octree_system};
use crate::plugins::environment::systems::voxels::queue_systems;
use crate::plugins::environment::systems::voxels::queue_systems::{enqueue_visible_chunks, process_chunk_queue};
use crate::plugins::environment::systems::voxels::render_chunks::rebuild_dirty_chunks;
use crate::plugins::environment::systems::voxels::lod::update_chunk_lods;
use crate::plugins::environment::systems::voxels::structure::{ChunkBudget, ChunkCullingCfg, ChunkQueue, SparseVoxelOctree, SpawnedChunks, PrevCameraChunk};

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Startup,
            (
                crate::plugins::environment::systems::camera_system::setup,
                crate::plugins::environment::systems::environment_system::setup.after(crate::plugins::environment::systems::camera_system::setup),
                crate::plugins::environment::systems::voxel_system::setup,
                execute_sphere_once.after(crate::plugins::environment::systems::voxel_system::setup),
            ),
        );

        let view_distance_chunks = 100;
        app.insert_resource(ChunkCullingCfg { view_distance_chunks });
        app.insert_resource(ChunkBudget { per_frame: 20 });
        app.init_resource::<PrevCameraChunk>();
        app.init_resource::<SphereGenerated>();
        app.add_systems(Update, log_mesh_count);
        app
            // ------------------------------------------------------------------------
            // resources
            // ------------------------------------------------------------------------
            .init_resource::<ChunkQueue>()
            .init_resource::<SpawnedChunks>()
            // ------------------------------------------------------------------------
            // frame update
            // ------------------------------------------------------------------------
            .add_systems(
                Update,
                (
                    /* ---------- culling & streaming ---------- */
                    enqueue_visible_chunks,
                    process_chunk_queue.after(enqueue_visible_chunks),
                    update_chunk_lods.after(process_chunk_queue),
                    rebuild_dirty_chunks .after(process_chunk_queue),
                    apply_sphere_result.after(rebuild_dirty_chunks),             // 4.  (re)mesh dirty chunks

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

fn log_mesh_count(meshes: Res<Assets<Mesh>>, time: Res<Time>) {
    if time.delta_secs_f64() as i32 % 5 == 0 {
        info!("meshes: {}", meshes.len());
    }
}

fn should_visualize_octree(octree_query: Query<&SparseVoxelOctree>,) -> bool {
    octree_query.single().show_wireframe
}

fn should_draw_grid(octree_query: Query<&SparseVoxelOctree>,) -> bool {
    octree_query.single().show_world_grid
}
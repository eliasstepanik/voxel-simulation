use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::{Chunk, ChunkLod, ChunkCullingCfg, SparseVoxelOctree, ChunkKey};
use super::lod_compute::{ChunkLodWorker, LodParams, MAX_LOD_CHUNKS};
use super::visible_chunks_compute::IVec3Pod;
use bevy_easy_compute::prelude::AppComputeWorker;

/// Update each chunk's LOD level based on its distance from the camera.
/// Chunks farther away get a higher LOD value (coarser mesh).
pub fn update_chunk_lods(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    chunks: Query<(Entity, &Chunk, &ChunkLod)>,
    mut chunks_mut: Query<&mut ChunkLod>,
    mut tree_q: Query<&mut SparseVoxelOctree>,
    mut worker: ResMut<AppComputeWorker<ChunkLodWorker>>,
    cfg: Res<ChunkCullingCfg>,
) {
    let cam_pos = cam_q.single().translation();
    let tree = tree_q.single();
    let max_depth = tree.max_depth - 1;
    let max_level = max_depth;
    let range_step = cfg.view_distance_chunks as f32 / (max_depth as f32 - 1.0);
    let centre = world_to_chunk(tree, cam_pos);
    drop(tree);

    let mut keys: Vec<IVec3Pod> = Vec::new();
    let mut entities: Vec<(Entity, ChunkKey)> = Vec::new();
    for (ent, chunk, _lod) in chunks.iter() {
        keys.push(IVec3Pod { x: chunk.key.0, y: chunk.key.1, z: chunk.key.2, _pad: 0 });
        entities.push((ent, chunk.key));
    }

    let count = keys.len().min(MAX_LOD_CHUNKS);
    worker.write_slice("keys_in", &keys[..count]);
    let params = LodParams {
        centre: IVec3Pod { x: centre.0, y: centre.1, z: centre.2, _pad: 0 },
        max_level,
        count: count as u32,
        range_step,
        _pad0: 0,
    };
    worker.write("params", &params);
    worker.execute();

    if !worker.ready() {
        return;
    }

    let results: Vec<u32> = worker.read_vec("lod_out");

    let mut tree = tree_q.single_mut();
    for ((ent, key), level) in entities.into_iter().zip(results.into_iter().take(count)) {
        if let Ok(mut lod) = chunks_mut.get_mut(ent) {
            if lod.0 != level {
                lod.0 = level;
                tree.dirty_chunks.insert(key);
            }
        }
    }
}

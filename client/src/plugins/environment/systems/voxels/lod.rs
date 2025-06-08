use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::chunk_center_world;
use crate::plugins::environment::systems::voxels::structure::{Chunk, ChunkLod, ChunkCullingCfg, SparseVoxelOctree};

/// Update each chunk's LOD level based on its distance from the camera.
/// Chunks farther away get a higher LOD value (coarser mesh).
pub fn update_chunk_lods(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    mut chunks: Query<(&Chunk, &mut ChunkLod)>,
    mut tree_q: Query<&mut SparseVoxelOctree>,
    cfg: Res<ChunkCullingCfg>,
) {
    let cam_pos = cam_q.single().translation();
    let max_depth;
    let range_step;
    {
        let tree = tree_q.single();
        max_depth = tree.max_depth;
        range_step = cfg.view_distance_chunks as f32 / max_depth as f32;
    }

    let mut changed = Vec::new();
    for (chunk, mut lod) in chunks.iter_mut() {
        let tree = tree_q.single();
        let center = chunk_center_world(&tree, chunk.key);
        let dist_chunks = cam_pos.distance(center)
            / (cfg.view_distance_chunks as f32);
        let mut level = (dist_chunks / range_step).floor() as u32;
        if level > max_depth { level = max_depth; }
        if lod.0 != level {
            lod.0 = level;
            changed.push(chunk.key);
        }
    }

    if !changed.is_empty() {
        let mut tree = tree_q.single_mut();
        for key in changed {
            tree.dirty_chunks.insert(key);
        }
    }
}

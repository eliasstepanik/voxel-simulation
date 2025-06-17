use crate::plugins::environment::systems::voxels::structure::{
    CHUNK_SIZE, Chunk, ChunkCullingCfg, ChunkLod, SparseVoxelOctree,
};
use bevy::prelude::*;

/// Update each chunk's LOD level based on its distance from the camera.
/// Chunks farther away get a higher LOD value (coarser mesh).
pub fn update_chunk_lods(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    mut chunks: Query<(&Chunk, &mut ChunkLod)>,
    mut tree_q: Query<&mut SparseVoxelOctree>,
    cfg: Res<ChunkCullingCfg>,
) {
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation();

    // Borrow the octree only once to avoid repeated query lookups
    let Ok(mut tree) = tree_q.get_single_mut() else {
        return;
    };
    let max_depth = tree.max_depth - 1;
    let range_step = cfg.view_distance_chunks as f32 / (max_depth as f32 - 1.0);
    let chunk_size = CHUNK_SIZE as f32 * tree.get_spacing_at_depth(max_depth);

    let mut changed = Vec::new();
    for (chunk, mut lod) in chunks.iter_mut() {
        let center = tree.chunk_center_world(chunk.key);
        let dist_chunks = cam_pos.distance(center) / chunk_size;
        let mut level = (dist_chunks / range_step).floor() as u32;
        if level > max_depth {
            level = max_depth;
        }
        if lod.0 != level {
            lod.0 = level;
            changed.push(chunk.key);
        }
    }

    for key in changed {
        tree.dirty_chunks.insert(key);
    }
}

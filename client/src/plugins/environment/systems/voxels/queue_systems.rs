use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;



/// enqueue chunks that *should* be visible but are not yet spawned
/// enqueue chunks that *should* be visible but are not yet spawned
pub fn enqueue_visible_chunks(
    mut queue      : ResMut<ChunkQueue>,
    spawned        : Res<SpawnedChunks>,
    mut prev_cam   : ResMut<PrevCameraChunk>,
    offsets        : Res<ChunkOffsets>,
    cfg            : Res<ChunkCullingCfg>,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
) {
    let tree     = tree_q.single();
    let cam_pos  = cam_q.single().translation();
    let centre   = world_to_chunk(tree, cam_pos);

    if prev_cam.0 == Some(centre) {
        return;
    }
    prev_cam.0 = Some(centre);

    for offset in &offsets.0 {
        let key = ChunkKey(centre.0 + offset.x, centre.1 + offset.y, centre.2 + offset.z);
        if spawned.0.contains_key(&key) { continue; }
        if queue.0.contains(&key)       { continue; }
        if !tree.occupied_chunks.contains(&key) { continue; }
        queue.0.push_back(key);
    }
}

/// move a limited number of keys from the queue into the octree’s dirty set
pub fn process_chunk_queue(
    mut queue   : ResMut<ChunkQueue>,
    budget      : Res<ChunkBudget>,
    mut tree_q  : Query<&mut SparseVoxelOctree>,
) {
    let mut tree = tree_q.single_mut();
    for _ in 0..budget.per_frame {
        if let Some(key) = queue.0.pop_front() {
            tree.dirty_chunks.insert(key);
        } else { break; }
    }
}
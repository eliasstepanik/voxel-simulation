use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;
use crate::plugins::environment::systems::voxels::visible_chunks_compute::{
    VisibleChunksWorker, VisibleParams, IVec3Pod, ChunkResult, VisibleChunkCount,
    MAX_VISIBLE_CHUNKS,
};
use bevy_easy_compute::prelude::*;



/// enqueue chunks that *should* be visible but are not yet spawned
/// enqueue chunks that *should* be visible but are not yet spawned
pub fn enqueue_visible_chunks(
    mut prev_cam   : ResMut<PrevCameraChunk>,
    cfg            : Res<ChunkCullingCfg>,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
    mut worker     : ResMut<AppComputeWorker<VisibleChunksWorker>>,
    mut count_res  : ResMut<VisibleChunkCount>,
) {
    let tree    = tree_q.single();
    let cam_pos = cam_q.single().translation();
    let centre  = world_to_chunk(tree, cam_pos);

    if prev_cam.0 == Some(centre) {
        return;
    }
    prev_cam.0 = Some(centre);

    let keys: Vec<IVec3Pod> = tree
        .occupied_chunks
        .iter()
        .map(|k| IVec3Pod { x: k.0, y: k.1, z: k.2, _pad: 0 })
        .collect();

    let count = keys.len().min(MAX_VISIBLE_CHUNKS);
    worker.write_slice("keys_in", &keys[..count]);

    let params = VisibleParams {
        centre: IVec3Pod { x: centre.0, y: centre.1, z: centre.2, _pad: 0 },
        radius: cfg.view_distance_chunks,
        count: count as u32,
        _pad0: 0,
    };
    worker.write("params", &params);
    worker.execute();
    count_res.0 = count;
}

/// move a limited number of keys from the queue into the octree’s dirty set
pub fn process_chunk_queue(
    mut queue   : ResMut<ChunkQueue>,
    budget      : Res<ChunkBudget>,
    mut tree_q  : Query<&mut SparseVoxelOctree>,
) {
    let mut tree = tree_q.single_mut();
    for _ in 0..budget.per_frame {
        if let Some(key) = queue.keys.pop_front() {
            queue.set.remove(&key);
            tree.dirty_chunks.insert(key);
        } else { break; }
    }
}

pub fn apply_visible_chunk_results(
    mut worker    : ResMut<AppComputeWorker<VisibleChunksWorker>>,
    mut queue     : ResMut<ChunkQueue>,
    spawned       : Res<SpawnedChunks>,
    count_res     : Res<VisibleChunkCount>,
) {
    if !worker.ready() {
        return;
    }

    let mut results: Vec<ChunkResult> = worker.read_vec("results");
    results.truncate(count_res.0);
    let mut keys: Vec<(ChunkKey, i32)> = results
        .into_iter()
        .filter_map(|r| {
            if r.dist2 < 0 { return None; }
            let key = ChunkKey(r.x, r.y, r.z);
            if spawned.0.contains_key(&key) { return None; }
            Some((key, r.dist2))
        })
        .collect();
    keys.sort_by_key(|(_, d)| *d);

    queue.keys.clear();
    queue.set.clear();
    for (key, _) in keys {
        queue.keys.push_back(key);
        queue.set.insert(key);
    }
}
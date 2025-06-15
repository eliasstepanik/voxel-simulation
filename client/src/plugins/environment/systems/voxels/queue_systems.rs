use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;
use bevy::prelude::*;
use rayon::prelude::*;

/// enqueue chunks that *should* be visible but are not yet spawned
/// enqueue chunks that *should* be visible but are not yet spawned
pub fn enqueue_visible_chunks(
    mut queue: ResMut<ChunkQueue>,
    spawned: Res<SpawnedChunks>,
    mut prev_cam: ResMut<PrevCameraChunk>,
    cfg: Res<ChunkCullingCfg>,
    cam_q: Query<&GlobalTransform, With<Camera>>,
    tree_q: Query<&SparseVoxelOctree>,
) {
    let Ok(tree) = tree_q.get_single() else {
        return;
    };
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation();
    let centre = world_to_chunk(tree, cam_pos);

    if prev_cam.0 == Some(centre) {
        return;
    }
    prev_cam.0 = Some(centre);

    let r = cfg.view_distance_chunks;

    let mut keys: Vec<(ChunkKey, i32)> = tree
        .occupied_chunks
        .par_iter()
        .filter_map(|key| {
            let dx = key.0 - centre.0;
            let dy = key.1 - centre.1;
            let dz = key.2 - centre.2;
            if dx.abs() > r || dy.abs() > r || dz.abs() > r {
                return None;
            }
            if spawned.0.contains_key(key) {
                return None;
            }
            Some((*key, dx * dx + dy * dy + dz * dz))
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

/// move a limited number of keys from the queue into the octree’s dirty set
pub fn process_chunk_queue(
    mut queue: ResMut<ChunkQueue>,
    budget: Res<ChunkBudget>,
    mut tree_q: Query<&mut SparseVoxelOctree>,
) {
    let Ok(mut tree) = tree_q.get_single_mut() else {
        return;
    };
    for _ in 0..budget.per_frame {
        if let Some(key) = queue.keys.pop_front() {
            queue.set.remove(&key);
            tree.dirty_chunks.insert(key);
        } else {
            break;
        }
    }
}

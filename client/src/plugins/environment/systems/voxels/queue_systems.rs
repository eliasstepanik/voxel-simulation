use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;



/// enqueue chunks that *should* be visible but are not yet spawned
/// enqueue chunks that *should* be visible but are not yet spawned
pub fn enqueue_visible_chunks(
    mut queue      : ResMut<ChunkQueue>,
    spawned        : Res<SpawnedChunks>,
    cfg            : Res<ChunkCullingCfg>,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
) {
    let tree     = tree_q.single();
    let cam_pos  = cam_q.single().translation();
    let centre   = world_to_chunk(tree, cam_pos);
    let r        = cfg.view_distance_chunks;

    // ------------------------------------------------------------------
    // 1. gather every *new* candidate chunk together with its distance²
    // ------------------------------------------------------------------
    let mut candidates: Vec<(i32 /*dist²*/, ChunkKey)> = Vec::new();

    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                let key = ChunkKey(centre.0 + dx, centre.1 + dy, centre.2 + dz);

                if spawned.0.contains_key(&key) { continue; }   // already spawned
                if queue.0.contains(&key)       { continue; }   // already queued
                if !tree.chunk_has_any_voxel(key) { continue; } // empty air

                let dist2 = dx*dx + dy*dy + dz*dz;              // squared distance
                candidates.push((dist2, key));
            }
        }
    }

    // ------------------------------------------------------------------
    // 2. sort by distance so nearest chunks enter the queue first
    // ------------------------------------------------------------------
    candidates.sort_by_key(|&(d2, _)| d2);

    // push into FIFO queue in that order
    for (_, key) in candidates {
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
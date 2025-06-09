use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;




    let r = cfg.view_distance_chunks;
    for key in &tree.occupied_chunks {
        let dx = key.0 - centre.0;
        let dy = key.1 - centre.1;
        let dz = key.2 - centre.2;
        if dx.abs() > r || dy.abs() > r || dz.abs() > r { continue; }
        if spawned.0.contains_key(key) { continue; }
        if queue.set.contains(key) { continue; }
        queue.keys.push_back(*key);
        queue.set.insert(*key);
        if let Some(key) = queue.keys.pop_front() {
            queue.set.remove(&key);
    cam_q        : Query<&GlobalTransform, With<Camera>>,
    tree_q       : Query<&SparseVoxelOctree>,
) {
    let tree    = tree_q.single();
    let cam_pos = cam_q.single().translation();
    let centre  = world_to_chunk(tree, cam_pos);

    // early-out if the camera stayed in the same chunk
    if prev_cam.0 == Some(centre) {
        return;
    }
    prev_cam.0 = Some(centre);

    for offset in &offsets.0 {
        let key = ChunkKey(
            centre.0 + offset.x,
            centre.1 + offset.y,
            centre.2 + offset.z,
        );

        if spawned.0.contains_key(&key)          { continue; }
        if queue.contains(&key)                  { continue; } // O(1) now
        if !tree.occupied_chunks.contains(&key) { continue; }

        queue.push(key); // O(1)
    }
}

/// move a limited number of keys from the queue into the octree’s dirty set
pub fn process_chunk_queue(
    mut queue  : ResMut<ChunkQueue>,
    budget     : Res<ChunkBudget>,
    mut tree_q : Query<&mut SparseVoxelOctree>,
) {
    let mut tree = tree_q.single_mut();

    for _ in 0..budget.per_frame {
        match queue.pop_front() {          // <-- use the new API
            Some(key) => { tree.dirty_chunks.insert(key); }
            None      => break,
        }
    }
}
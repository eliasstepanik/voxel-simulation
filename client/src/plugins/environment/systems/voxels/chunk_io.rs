use crate::plugins::environment::systems::voxels::structure::*;
use bevy::prelude::*;
use std::path::Path;

const CHUNK_DIR: &str = "chunks";

/// Save all dirty chunks to disk.
pub fn save_dirty_chunks_system(mut tree_q: Query<&mut SparseVoxelOctree>) {
    let Ok(mut tree) = tree_q.get_single_mut() else {
        return;
    };
    let _ = tree.save_dirty_chunks(Path::new(CHUNK_DIR));
}

/// Unload chunks that reached the maximum LOD distance.
pub fn unload_far_chunks(
    mut commands: Commands,
    mut tree_q: Query<&mut SparseVoxelOctree>,
    mut spawned: ResMut<SpawnedChunks>,
    chunks: Query<(Entity, &Chunk, &ChunkLod)>,
) {
    let Ok(mut tree) = tree_q.get_single_mut() else {
        return;
    };
    for (ent, chunk, lod) in chunks.iter() {
        if lod.0 == tree.max_depth - 1 {
            if let Err(e) = tree.save_chunk(chunk.key, Path::new(CHUNK_DIR)) {
                error!("failed to save chunk {:?}: {e}", chunk.key);
            }
            tree.unload_chunk(chunk.key);
            spawned.0.remove(&chunk.key);
            commands.entity(ent).despawn_recursive();
        }
    }
}

use crate::plugins::environment::systems::voxels::structure::*;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

/// despawn (or hide) every chunk entity whose centre is farther away than the
/// configured radius

pub fn despawn_distant_chunks(
    mut commands: Commands,
    cam_q: Query<&GlobalTransform, With<Camera>>,
    tree_q: Query<&SparseVoxelOctree>,
    mut spawned: ResMut<SpawnedChunks>,
    chunk_q: Query<(Entity, &Chunk, &Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cfg: Res<ChunkCullingCfg>,
) {
    let Ok(tree) = tree_q.get_single() else {
        return;
    };
    let Ok(cam_tf) = cam_q.get_single() else {
        return;
    };
    let cam = cam_tf.translation();
    let centre = tree.world_to_chunk(cam);

    for (ent, chunk, mesh3d, mat3d) in chunk_q.iter() {
        let ChunkKey(x, y, z) = chunk.key;
        if (x - centre.0).abs() > cfg.view_distance_chunks
            || (y - centre.1).abs() > cfg.view_distance_chunks
            || (z - centre.2).abs() > cfg.view_distance_chunks
        {
            // free assets – borrow, don't move
            meshes.remove(&mesh3d.0);
            materials.remove(&mat3d.0);

            commands.entity(ent).despawn_recursive();
            spawned.0.remove(&chunk.key);
        }
    }
}

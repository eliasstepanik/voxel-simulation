use std::collections::{HashMap, VecDeque};
use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;


/// despawn (or hide) every chunk entity whose centre is farther away than the
/// configured radius

pub fn despawn_distant_chunks(
    mut commands   : Commands,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
    mut spawned    : ResMut<SpawnedChunks>,
    chunk_q        : Query<(&Chunk, Option<&LodLevel>)>,
    cfg            : Res<ChunkCullingCfg>,
) {
    let tree   = tree_q.single();
    let cam    = cam_q.single().translation();
    let center = world_to_chunk(tree, cam);

    for (chunk, lod) in chunk_q.iter() {
        let range = if lod.map_or(0, |l| l.0) == 0 {
            cfg.view_distance_chunks
        } else {
            cfg.lod_distance_chunks
        };
        let ChunkKey(x, y, z) = chunk.key;
        if  (x - center.0).abs() > range ||
            (y - center.1).abs() > range ||
            (z - center.2).abs() > range {
            if let Some(ent) = spawned.0.remove(&chunk.key) {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}

/// remove low resolution chunks that moved within the high-res radius
pub fn replace_near_lod_chunks(
    mut commands   : Commands,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
    mut spawned    : ResMut<SpawnedChunks>,
    chunk_q        : Query<(Entity, &Chunk, Option<&LodLevel>)>,
    cfg            : Res<ChunkCullingCfg>,
) {
    let tree   = tree_q.single();
    let cam    = cam_q.single().translation();
    let center = world_to_chunk(tree, cam);

    for (entity, chunk, lod) in chunk_q.iter() {
        if let Some(lod) = lod {
            if lod.0 > 0 {
                let ChunkKey(x, y, z) = chunk.key;
                if (x - center.0).abs() <= cfg.view_distance_chunks &&
                   (y - center.1).abs() <= cfg.view_distance_chunks &&
                   (z - center.2).abs() <= cfg.view_distance_chunks {
                    commands.entity(entity).despawn_recursive();
                    spawned.0.remove(&chunk.key);
                }
            }
        }
    }
}
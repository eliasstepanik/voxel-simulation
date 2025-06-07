use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::structure::{ChunkKey, Voxel};

/// Component attached to the entity that owns the mesh of one chunk.
#[derive(Component)]
pub struct Chunk {
    pub key: ChunkKey,
    pub voxels: Vec<(IVec3, Voxel)>,   // local coords 0‥15
    pub dirty: bool,
}
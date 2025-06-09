use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::structure::{ChunkKey, SparseVoxelOctree, Voxel, CHUNK_POW, CHUNK_SIZE};

/// Component attached to the entity that owns the mesh of one chunk.

impl SparseVoxelOctree {
    pub fn chunk_has_any_voxel(&self, key: ChunkKey) -> bool {
        // world-space centre of the chunk
        let step  = self.get_spacing_at_depth(self.max_depth);
        let half  = self.size * 0.5;
        let centre = Vec3::new(
            (key.0 as f32 + 0.5) * CHUNK_SIZE as f32 * step - half,
            (key.1 as f32 + 0.5) * CHUNK_SIZE as f32 * step - half,
            (key.2 as f32 + 0.5) * CHUNK_SIZE as f32 * step - half,
        );

        // depth of the octree node that exactly matches one chunk
        let depth = self.max_depth.saturating_sub(CHUNK_POW);

        // normalised coordinates of that centre at the chosen depth
        let norm  = self.normalize_to_voxel_at_depth(centre, depth);

        // walk the tree down to that node …
        if let Some(node) =
            Self::get_node_at_depth(&self.root, norm.x, norm.y, norm.z, depth)
        {
            // … and ask whether that node or any child contains voxels
            return self.has_volume(node);
        }
        false
    }
}
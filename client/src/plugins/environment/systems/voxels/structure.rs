use std::collections::{HashMap, HashSet, VecDeque};
use bevy::color::Color;
use bevy::prelude::*;


/// Represents a single voxel with a color.
#[derive(Debug, Clone, Copy, Component, PartialEq, Default)]
pub struct Voxel {
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct DirtyVoxel {
    pub position: Vec3,
}

/// Represents a node in the sparse voxel octree.

#[derive(Debug, Component, Clone)]
pub struct OctreeNode {
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub voxel: Option<Voxel>,
    pub is_leaf: bool,
}
/// Represents the root of the sparse voxel octree.
/// Represents the root of the sparse voxel octree.
#[derive(Debug, Component)]
pub struct SparseVoxelOctree {

    pub root: OctreeNode,
    pub max_depth: u32,
    pub size: f32,
    pub show_wireframe: bool,
    pub show_world_grid: bool,
    pub show_chunks: bool,

    pub dirty: Vec<DirtyVoxel>,
    pub dirty_chunks: HashSet<ChunkKey>,
}

impl OctreeNode {
    /// Creates a new empty octree node.
    pub fn new() -> Self {
        Self {
            children: None,
            voxel: None,
            is_leaf: true,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxel.is_none() && self.children.is_none()
    }
}

impl Voxel {
    /// Creates a new empty octree node.
    pub fn new(color: Color) -> Self {
        Self {
            color,
        }
    }
}


pub const NEIGHBOR_OFFSETS: [(f32, f32, f32); 6] = [
    (-1.0, 0.0, 0.0), // Left
    (1.0, 0.0, 0.0),  // Right
    (0.0, -1.0, 0.0), // Down
    (0.0, 1.0, 0.0),  // Up
    (0.0, 0.0, -1.0), // Back
    (0.0, 0.0, 1.0),  // Front
];


#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

pub const CHUNK_SIZE: i32 = 16;         // 16×16×16 voxels
pub const CHUNK_POW : u32 = 4;

#[derive(Component)]
pub struct Chunk {
    pub key: ChunkKey,
    pub voxels: Vec<(IVec3, Voxel)>,   // local coords 0‥15
    pub dirty: bool,
    
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkKey(pub i32, pub i32, pub i32);


/// maximum amount of *new* chunk meshes we are willing to create each frame
#[derive(Resource)]
pub struct ChunkBudget {
    pub per_frame: usize,
}
impl Default for ChunkBudget {
    fn default() -> Self {
        Self { per_frame: 4 }      // tweak to taste
    }
}

/// FIFO queue with chunk keys that still need meshing
#[derive(Resource, Default)]
pub struct ChunkQueue(pub VecDeque<ChunkKey>);

/// map “which chunk key already has an entity in the world?”
#[derive(Resource, Default)]
pub struct SpawnedChunks(pub HashMap<ChunkKey, Entity>);

/// how big the cube around the player is, measured in chunks
#[derive(Resource)]
pub struct ChunkCullingCfg { pub view_distance_chunks: i32 }
impl Default for ChunkCullingCfg { fn default() -> Self { Self { view_distance_chunks: 6 } } }
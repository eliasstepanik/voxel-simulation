use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a single voxel with texture indices for each face.
#[derive(Debug, Clone, Copy, Component, PartialEq, Serialize, Deserialize)]
pub struct Voxel {
    /// Indexes into the texture atlas for the six faces in the order
    /// left, right, bottom, top, back, front.
    #[serde(default)]
    pub textures: [usize; 6],
}

impl Default for Voxel {
    fn default() -> Self {
        Self { textures: [0; 6] }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirtyVoxel {
    pub position: Vec3,
}

/// Represents a node in the sparse voxel octree.

#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct OctreeNode {
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub voxel: Option<Voxel>,
    pub is_leaf: bool,
}
/// Represents the root of the sparse voxel octree.
/// Represents the root of the sparse voxel octree.
#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct SparseVoxelOctree {
    pub root: OctreeNode,
    pub max_depth: u32,
    pub size: f32,
    pub show_wireframe: bool,
    pub show_world_grid: bool,

    #[serde(skip)]
    pub dirty: Vec<DirtyVoxel>,
    #[serde(skip)]
    pub dirty_chunks: HashSet<ChunkKey>,
    #[serde(skip)]
    pub occupied_chunks: HashSet<ChunkKey>,
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
    pub fn new(textures: [usize; 6]) -> Self {
        Self { textures }
    }

    /// Generate a voxel with a red top, black bottom and random colors on
    /// all remaining faces. Assumes the atlas uses index 0 for red, index 1
    /// for black and indices >=2 for random colors.
    pub fn random_sides() -> Self {
        let mut rng = rand::thread_rng();
        let mut textures = [0usize; 6];
        // Face order: left, right, bottom, top, back, front
        textures[3] = 0; // top is red
        textures[2] = 1; // bottom is black
        for &i in &[0usize, 1usize, 4usize, 5usize] {
            textures[i] = rng.gen_range(2..6);
        }
        Self { textures }
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

pub const CHUNK_SIZE: i32 = 16; // 16×16×16 voxels
pub const CHUNK_POW: u32 = 4;

#[derive(Component)]
pub struct Chunk {
    pub key: ChunkKey,
    pub voxels: Vec<(IVec3, Voxel)>, // local coords 0‥15
    pub dirty: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkLod(pub u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkKey(pub i32, pub i32, pub i32);

/// maximum amount of *new* chunk meshes we are willing to create each frame
#[derive(Resource)]
pub struct ChunkBudget {
    pub per_frame: usize,
}
impl Default for ChunkBudget {
    fn default() -> Self {
        Self { per_frame: 4 } // tweak to taste
    }
}

/// FIFO queue with chunk keys that still need meshing
#[derive(Resource, Default)]
pub struct ChunkQueue {
    pub keys: VecDeque<ChunkKey>,
    pub set: HashSet<ChunkKey>,
}

/// map “which chunk key already has an entity in the world?”
#[derive(Resource, Default)]
pub struct SpawnedChunks(pub HashMap<ChunkKey, Entity>);

/// how big the cube around the player is, measured in chunks
#[derive(Resource)]
pub struct ChunkCullingCfg {
    pub view_distance_chunks: i32,
}
impl Default for ChunkCullingCfg {
    fn default() -> Self {
        Self {
            view_distance_chunks: 6,
        }
    }
}

#[derive(Resource, Default)]
pub struct PrevCameraChunk(pub Option<ChunkKey>);

#[derive(Resource, Clone)]
pub struct ChunkOffsets(pub Vec<IVec3>);

impl ChunkOffsets {
    pub fn new(radius: i32) -> Self {
        let mut offsets = Vec::new();
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    offsets.push(IVec3::new(dx, dy, dz));
                }
            }
        }
        offsets.sort_by_key(|v| v.x * v.x + v.y * v.y + v.z * v.z);
        Self(offsets)
    }
}

/// Pool reused when constructing chunk meshes. Reusing the backing
/// storage avoids frequent allocations when rebuilding many chunks.
#[derive(Resource, Default)]
pub struct MeshBufferPool {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MeshBufferPool {
    /// Clears all buffers while keeping the allocated capacity.
    pub fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.uvs.clear();
        self.indices.clear();
    }
}

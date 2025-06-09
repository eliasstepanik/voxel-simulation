use std::collections::{HashMap, HashSet, VecDeque};
use bevy::color::Color;
use bevy::prelude::*;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let [r, g, b, a] = color.to_linear().to_f32_array();
    [r, g, b, a].serialize(serializer)
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let arr: [f32; 4] = Deserialize::deserialize(deserializer)?;
    Ok(Color::linear_rgba(arr[0], arr[1], arr[2], arr[3]))
}


/// Represents a single voxel with a color.
#[derive(Debug, Clone, Copy, Component, PartialEq, Default, Serialize, Deserialize)]
pub struct Voxel {
    #[serde(serialize_with = "serialize_color", deserialize_with = "deserialize_color")]
    pub color: Color,
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
        Self { per_frame: 4 }      // tweak to taste
    }
}

/// FIFO queue with chunk keys that still need meshing
#[derive(Resource, Default)]
pub struct ChunkQueue {
    pub keys: VecDeque<ChunkKey>,
    pub set:  HashSet<ChunkKey>,
}

/// map “which chunk key already has an entity in the world?”
#[derive(Resource, Default)]
pub struct SpawnedChunks(pub HashMap<ChunkKey, Entity>);

/// how big the cube around the player is, measured in chunks
#[derive(Resource)]
pub struct ChunkCullingCfg { pub view_distance_chunks: i32 }
impl Default for ChunkCullingCfg { fn default() -> Self { Self { view_distance_chunks: 6 } } }

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

use bevy::prelude::*;
use bevy_easy_compute::prelude::*;
use bytemuck::{Pod, Zeroable};
use super::structure::CHUNK_SIZE;

const MAX_FACES: usize = (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * 6;
pub const MAX_VERTICES: usize = MAX_FACES * 4;
pub const MAX_INDICES: usize = MAX_FACES * 6;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct MeshParams {
    pub origin: [f32; 3],
    pub step: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType, Default)]
pub struct MeshCounts {
    pub vertex_count: u32,
    pub index_count: u32,
}

#[derive(TypePath)]
struct MeshShader;

impl ComputeShader for MeshShader {
    fn shader() -> ShaderRef {
        "shaders/chunk_mesh.wgsl".into()
    }
}

#[derive(Resource)]
pub struct ChunkMeshWorker;

impl ComputeWorker for ChunkMeshWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = MeshParams { origin: [0.0; 3], step: 1.0 };
        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_staging("voxels", &[0u32; CHUNK_SIZE as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize])
            .add_staging("positions", &[[0.0_f32; 3]; MAX_VERTICES])
            .add_staging("normals", &[[0.0_f32; 3]; MAX_VERTICES])
            .add_staging("uvs", &[[0.0_f32; 2]; MAX_VERTICES])
            .add_staging("indices", &[0u32; MAX_INDICES])
            .add_staging("counts", &MeshCounts::default())
            .add_pass::<MeshShader>([1, 1, 1], &["params", "voxels", "positions", "normals", "uvs", "indices", "counts"])
            .one_shot()
            .synchronous()
            .build()
    }
}

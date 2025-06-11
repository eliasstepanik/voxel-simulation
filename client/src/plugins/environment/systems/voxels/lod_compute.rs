use bevy::prelude::*;
use bevy_easy_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::visible_chunks_compute::IVec3Pod;

pub const MAX_LOD_CHUNKS: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct LodParams {
    pub centre: IVec3Pod,
    pub max_level: u32,
    pub count: u32,
    pub range_step: f32,
    pub _pad0: u32,
}

#[derive(TypePath)]
struct LodShader;

impl ComputeShader for LodShader {
    fn shader() -> ShaderRef {
        "shaders/chunk_lod.wgsl".into()
    }
}

#[derive(Resource)]
pub struct ChunkLodWorker;

impl ComputeWorker for ChunkLodWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = LodParams {
            centre: IVec3Pod { x: 0, y: 0, z: 0, _pad: 0 },
            max_level: 0,
            count: 0,
            range_step: 1.0,
            _pad0: 0,
        };
        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_staging("keys_in", &[IVec3Pod { x: 0, y: 0, z: 0, _pad: 0 }; MAX_LOD_CHUNKS])
            .add_staging("lod_out", &[0u32; MAX_LOD_CHUNKS])
            .add_pass::<LodShader>([((MAX_LOD_CHUNKS as u32 + 63) / 64), 1, 1], &["params", "keys_in", "lod_out"])
            .one_shot()
            .synchronous()
            .build()
    }
}

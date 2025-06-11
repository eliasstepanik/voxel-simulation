use bevy::prelude::*;
use bevy_easy_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

pub const MAX_VISIBLE_CHUNKS: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct IVec3Pod {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub _pad: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct VisibleParams {
    pub centre: IVec3Pod,
    pub radius: i32,
    pub count: u32,
    pub _pad0: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
pub struct ChunkResult {
    pub key: IVec3Pod,
    pub dist2: i32,
}

#[derive(TypePath)]
struct VisibleShader;

impl ComputeShader for VisibleShader {
    fn shader() -> ShaderRef {
        "shaders/visible_chunks.wgsl".into()
    }
}

#[derive(Resource)]
pub struct VisibleChunksWorker;

impl ComputeWorker for VisibleChunksWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let default_params = VisibleParams {
            centre: IVec3Pod { x: 0, y: 0, z: 0, _pad: 0 },
            radius: 0,
            count: 0,
            _pad0: 0,
        };
        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &default_params)
            .add_staging("keys_in", &[IVec3Pod { x: 0, y: 0, z: 0, _pad: 0 }; MAX_VISIBLE_CHUNKS])
            .add_staging("results", &[ChunkResult { key: IVec3Pod { x: 0, y: 0, z: 0, _pad: 0 }, dist2: 0 }; MAX_VISIBLE_CHUNKS])
            .add_pass::<VisibleShader>([((MAX_VISIBLE_CHUNKS as u32 + 63) / 64), 1, 1], &["params", "keys_in", "results"])
            .one_shot()
            .synchronous()
            .build()
    }
}

#[derive(Resource, Default)]
pub struct VisibleChunkCount(pub usize);

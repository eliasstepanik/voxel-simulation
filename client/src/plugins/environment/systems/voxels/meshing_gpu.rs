use bevy::prelude::*;
use bevy_app_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

use super::structure::{MeshBufferPool, SparseVoxelOctree};

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Pod, Zeroable, Default)]
pub struct Params {
    pub origin: Vec3,
    pub step: f32,
    pub axis: u32,
    pub dir: i32,
    pub slice: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Pod, Zeroable, Default)]
pub struct VertexGpu {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(TypePath)]
struct GreedyMeshingShader;

impl ComputeShader for GreedyMeshingShader {
    fn shader() -> ShaderRef {
        "shaders/greedy_meshing.wgsl".into()
    }
}

/// GPU worker executing greedy meshing for chunks.
#[derive(Resource)]
pub struct GpuMeshingWorker;

impl ComputeWorker for GpuMeshingWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        AppComputeWorkerBuilder::new(world)
            .add_storage::<u32>("voxels", &[0u32; 1])
            .add_uniform("params", &Params::default())
            .add_storage::<VertexGpu>("vertices", &[VertexGpu::default(); 1])
            .add_storage::<u32>("indices", &[0u32; 1])
            .add_storage::<u32>("counts", &[0u32; 2])
            .add_pass::<GreedyMeshingShader>(
                [1, 1, 1],
                &["voxels", "params", "vertices", "indices", "counts"],
            )
            .one_shot()
            .build()
    }
}

/// Placeholder system that will dispatch the compute worker for dirty chunks.
pub fn queue_gpu_meshing(
    mut worker: ResMut<AppComputeWorker<GpuMeshingWorker>>,
    _octrees: Query<&SparseVoxelOctree>,
    _pool: ResMut<MeshBufferPool>,
) {
    if !worker.ready() {
        return;
    }
    // TODO: populate the worker buffers with chunk data before dispatching.
    worker.execute();
}

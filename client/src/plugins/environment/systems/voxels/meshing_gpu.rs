use bevy::prelude::*;
use bevy_app_compute::prelude::*;

use super::structure::{CHUNK_SIZE, MeshBufferPool, SparseVoxelOctree};

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default)]
pub struct Params {
    pub origin: Vec3,
    pub step: f32,
    pub axis: u32,
    pub dir: i32,
    pub slice: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default)]
pub struct VertexGpu {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

const MAX_VOXELS: usize = (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);
const MAX_QUADS: usize = MAX_VOXELS * 6;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;

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
        // Allocate large temporary arrays on the heap to avoid stack overflows
        let voxels = Box::new([0u32; MAX_VOXELS]);
        let vertices = Box::new([VertexGpu::default(); MAX_VERTICES]);
        let indices = Box::new([0u32; MAX_INDICES]);

        AppComputeWorkerBuilder::new(world)
            .add_storage("voxels", voxels.as_ref())
            .add_uniform("params", &Params::default())
            .add_storage("vertices", vertices.as_ref())
            .add_storage("indices", indices.as_ref())
            .add_storage("counts", &[0u32; 2])
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

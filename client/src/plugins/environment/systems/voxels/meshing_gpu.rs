use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::asset::RenderAssetUsages;
use bevy_app_compute::prelude::*;
use super::structure::{Voxel, CHUNK_SIZE};

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    pub origin: Vec3,
    pub step: f32,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexGpu {
    pub pos: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub uv: [f32; 2],
    pub _pad2: [f32; 2],
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Counts {
    pub verts: u32,
    pub indices: u32,
    pub _pad: [u32; 2],
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
            .add_storage("voxels", &[0u32; 1])
            .add_uniform("params", &Params::default())
            .add_storage("vertices", &[VertexGpu::default(); 1])
            .add_storage("indices", &[0u32; 1])
            .add_storage("counts", &[Counts::default()])
            .add_pass::<GreedyMeshingShader>(
                [1, 1, 1],
                &["voxels", "params", "vertices", "indices", "counts"],
            )
            .one_shot()
            .build()
    }
}

/// Placeholder system that will dispatch the compute worker for dirty chunks.
const N: usize = CHUNK_SIZE as usize;
const MAX_VERTS: usize = N * N * N * 4 * 6 / 2; // generous upper bound
const MAX_INDICES: usize = N * N * N * 6 * 6 / 2;

pub fn mesh_chunk_gpu(
    worker: &mut AppComputeWorker<GpuMeshingWorker>,
    buffer: &[[[Option<Voxel>; N]; N]; N],
    origin: Vec3,
    step: f32,
) -> Option<Mesh> {
    if !worker.ready() {
        return None;
    }

    // Flatten voxel data (1 = filled, 0 = empty).
    let mut voxels = [0u32; N * N * N];
    let mut i = 0;
    for x in 0..N {
        for y in 0..N {
            for z in 0..N {
                voxels[i] = if buffer[x][y][z].is_some() { 1 } else { 0 };
                i += 1;
            }
        }
    }

    // Preallocate output buffers.
    let verts_empty = vec![VertexGpu::default(); MAX_VERTS];
    let indices_empty = vec![0u32; MAX_INDICES];
    let counts = Counts::default();

    worker.write_slice("voxels", &voxels);
    worker.write_slice("vertices", &verts_empty);
    worker.write_slice("indices", &indices_empty);
    worker.write_slice("counts", &[counts]);

    let params = Params { origin, step };
    worker.write("params", &params);

    worker.execute();

    let counts: Counts = worker.read("counts");
    let v_count = counts.verts as usize;
    let i_count = counts.indices as usize;

    if i_count == 0 {
        return None;
    }

    let verts: Vec<VertexGpu> = worker.read_vec("vertices");
    let indices: Vec<u32> = worker.read_vec("indices");

    let mut positions = Vec::with_capacity(v_count);
    let mut normals = Vec::with_capacity(v_count);
    let mut uvs = Vec::with_capacity(v_count);

    for v in verts.into_iter().take(v_count) {
        positions.push([v.pos[0], v.pos[1], v.pos[2]]);
        normals.push([v.normal[0], v.normal[1], v.normal[2]]);
        uvs.push([v.uv[0], v.uv[1]]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uvs),
    );
    mesh.insert_indices(Indices::U32(indices.into_iter().take(i_count).collect()));
    Some(mesh)
}

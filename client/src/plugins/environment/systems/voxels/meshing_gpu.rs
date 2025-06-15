use bevy::prelude::*;
use bevy_app_compute::prelude::*;

use bytemuck::{Pod, Zeroable};

use std::collections::HashMap;

use super::structure::{
    Chunk, ChunkLod, ChunkKey, MeshBufferPool, SpawnedChunks, SparseVoxelOctree, CHUNK_SIZE,
};
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::atlas::VoxelTextureAtlas;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use big_space::prelude::{GridCell, Mesh3d, MeshMaterial3d};

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, Pod, Zeroable)]
pub struct Params {
    pub origin: Vec3,
    pub step: f32,
    pub axis: u32,
    pub dir: i32,
    pub slice: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, Pod, Zeroable)]
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
            .add_staging("vertices", vertices.as_ref())
            .add_staging("indices", indices.as_ref())
            .add_staging("counts", &[0u32; 2])
            .add_pass::<GreedyMeshingShader>(
                [1, 1, 1],
                &["voxels", "params", "vertices", "indices", "counts"],
            )
            .one_shot()
            .build()
    }
}

/// Tracks the chunk currently being meshed on the GPU.
#[derive(Resource, Default)]
pub struct GpuMeshingJob {
    pub key: Option<ChunkKey>,
    pub origin: Vec3,
    pub step: f32,
    pub lod: u32,
}

/// Dispatch greedy meshing on the GPU one chunk at a time.
#[allow(clippy::too_many_arguments)]
pub fn queue_gpu_meshing(
    mut commands: Commands,
    mut worker: ResMut<AppComputeWorker<GpuMeshingWorker>>,
    mut job: ResMut<GpuMeshingJob>,
    mut octrees: Query<&mut SparseVoxelOctree>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_q: Query<(Entity, &Chunk, &Mesh3d, &MeshMaterial3d<StandardMaterial>, &ChunkLod)>,
    mut spawned: ResMut<SpawnedChunks>,
    mut pool: ResMut<MeshBufferPool>,
    root: Res<RootGrid>,
    atlas: Res<VoxelTextureAtlas>,
) {
    let Ok(mut tree) = octrees.get_single_mut() else { return; };

    let existing: HashMap<ChunkKey, (Entity, Handle<Mesh>, Handle<StandardMaterial>, u32)> =
        chunk_q
            .iter()
            .map(|(e, c, m, mat, lod)| (c.key, (e, m.0.clone(), mat.0.clone(), lod.0)))
            .collect();

    // If a job finished, read back the mesh and spawn/update the chunk.
    if let Some(key) = job.key {
        if worker.ready() {
            let vertices: Vec<VertexGpu> = worker.read_vec("vertices");
            let indices: Vec<u32> = worker.read_vec("indices");
            let counts: Vec<u32> = worker.read_vec("counts");
            let vert_count = counts.get(0).copied().unwrap_or(0) as usize;
            let index_count = counts.get(1).copied().unwrap_or(0) as usize;

            if index_count == 0 {
                if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                    meshes.remove(&mesh_h);
                    commands.entity(ent).despawn_recursive();
                    spawned.0.remove(&key);
                }
            } else {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                let positions: Vec<[f32; 3]> = vertices[..vert_count]
                    .iter()
                    .map(|v| [v.pos.x, v.pos.y, v.pos.z])
                    .collect();
                let normals: Vec<[f32; 3]> = vertices[..vert_count]
                    .iter()
                    .map(|v| [v.normal.x, v.normal.y, v.normal.z])
                    .collect();
                let uvs: Vec<[f32; 2]> = vertices[..vert_count]
                    .iter()
                    .map(|v| [v.uv.x, v.uv.y])
                    .collect();
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
                mesh.insert_indices(Indices::U32(indices[..index_count].to_vec()));

                if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                    if let Some(m) = meshes.get_mut(&mesh_h) {
                        *m = mesh;
                    }
                    spawned.0.insert(key, ent);
                } else {
                    let mesh_h = meshes.add(mesh);
                    let mat_h = materials.add(StandardMaterial {
                        base_color_texture: Some(atlas.handle.clone()),
                        ..Default::default()
                    });
                    commands.entity(root.0).with_children(|p| {
                        let e = p
                            .spawn((
                                Mesh3d::from(mesh_h.clone()),
                                MeshMaterial3d(mat_h.clone()),
                                Transform::default(),
                                GridCell::ZERO,
                                Chunk { key, voxels: Vec::new(), dirty: false },
                                ChunkLod(job.lod),
                            ))
                            .id();
                        spawned.0.insert(key, e);
                    });
                }
            }

            job.key = None;
            worker.write_slice("counts", &[0u32, 0u32]);
        }
        return;
    }

    // No active job - start meshing the next dirty chunk.
    if !worker.ready() {
        return;
    }

    let Some(&key) = tree.dirty_chunks.iter().next() else { return; };

    let lod = existing.get(&key).map(|v| v.3).unwrap_or(0);
    let mut buf = [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    let half = tree.size * 0.5;
    let step = tree.get_spacing_at_depth(tree.max_depth);
    let origin = Vec3::new(
        key.0 as f32 * CHUNK_SIZE as f32 * step - half,
        key.1 as f32 * CHUNK_SIZE as f32 * step - half,
        key.2 as f32 * CHUNK_SIZE as f32 * step - half,
    );

    let mult = 1 << lod;
    for gx in (0..CHUNK_SIZE).step_by(mult as usize) {
        for gy in (0..CHUNK_SIZE).step_by(mult as usize) {
            for gz in (0..CHUNK_SIZE).step_by(mult as usize) {
                let center = origin
                    + Vec3::new(
                        (gx + mult / 2) as f32 * step,
                        (gy + mult / 2) as f32 * step,
                        (gz + mult / 2) as f32 * step,
                    );
                if let Some(v) = tree.get_voxel_at_world_coords(center) {
                    for lx in 0..mult {
                        for ly in 0..mult {
                            for lz in 0..mult {
                                let ix = gx + lx;
                                let iy = gy + ly;
                                let iz = gz + lz;
                                if ix < CHUNK_SIZE && iy < CHUNK_SIZE && iz < CHUNK_SIZE {
                                    buf[ix as usize][iy as usize][iz as usize] = Some(*v);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tree.dirty_chunks.remove(&key);

    let mut vox = vec![0u32; MAX_VOXELS];
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let idx = (x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z) as usize;
                vox[idx] = if buf[x as usize][y as usize][z as usize].is_some() { 1 } else { 0 };
            }
        }
    }

    worker.write_slice("voxels", &vox);
    worker.write(
        "params",
        &Params { origin, step, axis: 0, dir: 0, slice: 0, _pad: 0 },
    );
    worker.write_slice("counts", &[0u32, 0u32]);
    worker.execute();

    job.key = Some(key);
    job.origin = origin;
    job.step = step;
    job.lod = lod;
}

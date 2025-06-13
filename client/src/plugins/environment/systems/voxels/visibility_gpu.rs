use bevy::prelude::*;
use bevy_app_compute::prelude::*;

use super::structure::{ChunkCullingCfg, ChunkQueue, PrevCameraChunk, SpawnedChunks, SparseVoxelOctree, ChunkKey};
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;

#[repr(C)]
#[derive(ShaderType, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    pub centre_radius: [i32; 4],
    pub count: u32,
    pub _pad: u32,
}

#[derive(TypePath)]
struct VisibilityShader;

impl ComputeShader for VisibilityShader {
    fn shader() -> ShaderRef {
        "shaders/chunk_visibility.wgsl".into()
    }
}

#[derive(Resource)]
pub struct GpuVisibilityWorker;

impl ComputeWorker for GpuVisibilityWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        AppComputeWorkerBuilder::new(world)
            .add_storage::<[IVec3; 1]>("occupied", &[IVec3::ZERO; 1])
            .add_storage::<[u32; 1]>("spawned", &[0u32; 1])
            .add_rw_storage::<[IVec3; 1]>("out_keys", &[IVec3::ZERO; 1])
            .add_rw_storage::<u32>("out_count", &0u32)
            .add_uniform("params", &Params::default())
            .add_pass::<VisibilityShader>([1024, 1, 1], &["occupied", "spawned", "out_keys", "out_count", "params"])
            .one_shot()
            .build()
    }
}

/// GPU-driven implementation of `enqueue_visible_chunks`.
pub fn enqueue_visible_chunks_gpu(
    mut worker: ResMut<AppComputeWorker<GpuVisibilityWorker>>,
    tree_q: Query<&SparseVoxelOctree>,
    cam_q: Query<&GlobalTransform, With<Camera>>,
    spawned: Res<SpawnedChunks>,
    mut prev_cam: ResMut<PrevCameraChunk>,
    cfg: Res<ChunkCullingCfg>,
    mut queue: ResMut<ChunkQueue>,
) {
    let Ok(tree) = tree_q.get_single() else { return };
    let Ok(cam_tf) = cam_q.get_single() else { return };
    let cam_pos = cam_tf.translation();
    let centre_key = world_to_chunk(tree, cam_pos);
    if prev_cam.0 == Some(centre_key) { return; }
    prev_cam.0 = Some(centre_key);

    if !worker.ready() { return; }

    let occupied_keys: Vec<ChunkKey> = tree.occupied_chunks.iter().copied().collect();
    let occupied: Vec<IVec3> = occupied_keys
        .iter()
        .map(|k| IVec3::new(k.0, k.1, k.2))
        .collect();
    let mut spawned_flags = Vec::with_capacity(occupied_keys.len());
    for key in &occupied_keys {
        spawned_flags.push(if spawned.0.contains_key(key) { 1u32 } else { 0u32 });
    }
    worker.write_slice("occupied", &occupied);
    worker.write_slice("spawned", &spawned_flags);
    worker.write_slice("out_keys", &vec![IVec3::ZERO; occupied.len()]);
    worker.write("out_count", &0u32);

    let params = Params {
        centre_radius: [
            centre_key.0,
            centre_key.1,
            centre_key.2,
            cfg.view_distance_chunks,
        ],
        count: occupied.len() as u32,
        _pad: 0,
    };
    worker.write("params", &params);

    worker.execute();

    let count: u32 = worker.read("out_count");
    let keys: Vec<IVec3> = worker.read_vec("out_keys");
    queue.keys.clear();
    queue.set.clear();
    for key in keys.into_iter().take(count as usize) {
        let ck = ChunkKey(key.x, key.y, key.z);
        queue.keys.push_back(ck);
        queue.set.insert(ck);
    }
}

use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::RenderApp;

use super::structure::{MeshBufferPool, SparseVoxelOctree};

/// Runs greedy meshing on the GPU.
pub struct GpuMeshingPlugin;

impl Plugin for GpuMeshingPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GpuMeshingPipeline>()
            .add_systems(Render, queue_gpu_meshing);
    }
}

#[derive(Resource)]
pub struct GpuMeshingPipeline {
    pub pipeline: CachedComputePipelineId,
    pub layout: BindGroupLayout,
}

impl FromWorld for GpuMeshingPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader: Handle<Shader> = asset_server.load("shaders/greedy_meshing.wgsl");
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("meshing_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_descriptor = ComputePipelineDescriptor {
            label: Some("meshing_pipeline".into()),
            layout: vec![layout.clone()],
            shader,
            shader_defs: vec![],
            entry_point: "main".into(),
        };
        let render_queue = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(render_queue, &pipeline_descriptor);
        Self { pipeline, layout }
    }
}

/// System that dispatches the compute shader for dirty chunks.
fn queue_gpu_meshing(
    _octrees: Query<&SparseVoxelOctree>,
    _pool: ResMut<MeshBufferPool>,
    _pipeline: Res<GpuMeshingPipeline>,
) {
    // TODO: upload voxel buffers and dispatch compute passes per chunk.
}

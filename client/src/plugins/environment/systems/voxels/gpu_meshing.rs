use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::renderer::RenderQueue;


pub struct GpuMesher {
    pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for GpuMesher {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let shader = Shader::from_wgsl(include_str!("../../../../../assets/shaders/chunk_mesh.wgsl"));
        let shader_module = render_device.create_shader_module(ShaderModuleDescriptor {
            label: Some("chunk mesh shader"),
            source: ShaderSource::Wgsl(shader.to_string().into()),
        });

        let bind_group_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk mesh layout"),
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
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("chunk mesh pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = render_device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("chunk mesh pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

        Self {
            pipeline: pipeline.into(),
            bind_group_layout,
        }
    }
}

impl GpuMesher {
    pub fn mesh_chunk(&self, device: &RenderDevice, queue: &RenderQueue, voxels: &[u32], output: &Buffer) {
        let voxel_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel buffer"),
            contents: bytemuck::cast_slice(voxels),
            usage: BufferUsages::STORAGE,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("chunk mesh bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: voxel_buffer.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: output.as_entire_binding() },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("chunk mesh encoder") });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("chunk mesh pass") });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((voxels.len() as u32) + 63) / 64;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

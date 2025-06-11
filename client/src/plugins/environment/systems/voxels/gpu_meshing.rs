use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::render_resource::ComputePipelineDescriptor as RawComputePipelineDescriptor;
use bevy::render::renderer::RenderDevice;
use bevy::render::renderer::RenderQueue;
use bevy::render::render_resource::Maintain;
use crossbeam_channel::bounded;


pub struct GpuMesher {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for GpuMesher {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let shader_module = render_device.create_shader_module(ShaderModuleDescriptor {
            label: Some("chunk mesh shader"),
            source: ShaderSource::Wgsl(
                include_str!("../../../../../assets/shaders/chunk_mesh.wgsl").into(),
            ),
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

        let pipeline = render_device.create_compute_pipeline(&RawComputePipelineDescriptor {
            label: Some("chunk mesh pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl GpuMesher {
    /// Compute the face visibility mask for the given voxel array using the GPU.
    pub fn compute_face_mask(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
        voxels: &[u32],
    ) -> Vec<u32> {
        let output_size = (voxels.len() * std::mem::size_of::<u32>()) as u64;
        let output = device.create_buffer(&BufferDescriptor {
            label: Some("face mask buffer"),
            size: output_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
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
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("chunk mesh pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((voxels.len() as u32) + 63) / 64;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(encoder.finish()));

        let buffer_slice = output.slice(..);
        let (s, r) = bounded(1);
        buffer_slice.map_async(MapMode::Read, move |res| { let _ = s.send(res); });
        device.poll(Maintain::Wait);
        let _ = r.recv();
        let data = buffer_slice.get_mapped_range().to_vec();
        output.unmap();
        bytemuck::cast_slice(&data).to_vec()
    }
}

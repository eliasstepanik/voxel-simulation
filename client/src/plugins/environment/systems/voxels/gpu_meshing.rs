use bevy::prelude::*;
use bevy::render::render_resource::ComputePipelineDescriptor as RawComputePipelineDescriptor;
use bevy::render::render_resource::Maintain;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::renderer::RenderQueue;
use crossbeam_channel::bounded;

pub struct GpuMesher {
    mask_pipeline: ComputePipeline,
    mesh_pipeline: ComputePipeline,
    mask_layout: BindGroupLayout,
    mesh_layout: BindGroupLayout,
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

        // Layout for face counting
        let mask_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk mask layout"),
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
            ],
        });
        // Layout for mesh generation
        let mesh_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("chunk mesh layout"),
            entries: &[
                // voxels
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
                // face mask
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // prefix
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // positions output
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
                // normals output
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
                // params uniform
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Pipelines
        let mask_pipeline_layout =
            render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("mask pipeline layout"),
                bind_group_layouts: &[&mask_layout],
                push_constant_ranges: &[],
            });

        let mask_pipeline = render_device.create_compute_pipeline(&RawComputePipelineDescriptor {
            label: Some("mask pipeline"),
            layout: Some(&mask_pipeline_layout),
            module: &shader_module,
            entry_point: Some("count_faces"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let mesh_pipeline_layout =
            render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("mesh pipeline layout"),
                bind_group_layouts: &[&mesh_layout],
                push_constant_ranges: &[],
            });

        let mesh_pipeline = render_device.create_compute_pipeline(&RawComputePipelineDescriptor {
            label: Some("mesh pipeline"),
            layout: Some(&mesh_pipeline_layout),
            module: &shader_module,
            entry_point: Some("generate_mesh"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            mask_pipeline,
            mesh_pipeline,
            mask_layout,
            mesh_layout,
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
    ) -> (Vec<u32>, Vec<u32>) {
        let output_size = (voxels.len() * std::mem::size_of::<u32>()) as u64;
        let output = device.create_buffer(&BufferDescriptor {
            label: Some("face mask buffer"),
            size: output_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let count_output = device.create_buffer(&BufferDescriptor {
            label: Some("face count buffer"),
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
            label: Some("chunk mask bind group"),
            layout: &self.mask_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: voxel_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: count_output.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("chunk mesh encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("chunk mesh pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.mask_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((voxels.len() as u32) + 63) / 64;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(encoder.finish()));

        let buffer_slice = output.slice(..);
        let count_slice = count_output.slice(..);
        let (s, r) = bounded(1);
        buffer_slice.map_async(MapMode::Read, move |res| {
            let _ = s.send(res);
        });
        let (s2, r2) = bounded(1);
        count_slice.map_async(MapMode::Read, move |res| {
            let _ = s2.send(res);
        });
        device.poll(Maintain::Wait);
        let _ = r.recv();
        let _ = r2.recv();
        let data_mask = buffer_slice.get_mapped_range().to_vec();
        let data_count = count_slice.get_mapped_range().to_vec();
        output.unmap();
        count_output.unmap();
        (
            bytemuck::cast_slice(&data_mask).to_vec(),
            bytemuck::cast_slice(&data_count).to_vec(),
        )
    }

    /// Generate vertex positions and normals for visible faces using the GPU.
    pub fn generate_mesh(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
        voxels: &[u32],
        mask: &[u32],
        prefix: &[u32],
        vertex_count: usize,
        origin: Vec3,
        step: f32,
    ) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let buffer_size = (vertex_count * std::mem::size_of::<[f32; 3]>()) as u64;
        let positions = device.create_buffer(&BufferDescriptor {
            label: Some("mesh positions"),
            size: buffer_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let normals = device.create_buffer(&BufferDescriptor {
            label: Some("mesh normals"),
            size: buffer_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let voxel_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("voxel buffer"),
            contents: bytemuck::cast_slice(voxels),
            usage: BufferUsages::STORAGE,
        });
        let mask_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("mask buffer"),
            contents: bytemuck::cast_slice(mask),
            usage: BufferUsages::STORAGE,
        });
        let prefix_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("prefix buffer"),
            contents: bytemuck::cast_slice(prefix),
            usage: BufferUsages::STORAGE,
        });
        let params = [origin.x, origin.y, origin.z, step];
        let param_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("params"),
            contents: bytemuck::cast_slice(&params),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("mesh bind group"),
            layout: &self.mesh_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: voxel_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: mask_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: prefix_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: positions.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: normals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: param_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("generate mesh encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("generate mesh pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.mesh_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((voxels.len() as u32) + 63) / 64;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(Some(encoder.finish()));

        let slice_p = positions.slice(..);
        let slice_n = normals.slice(..);
        let (s1, r1) = bounded(1);
        slice_p.map_async(MapMode::Read, move |res| {
            let _ = s1.send(res);
        });
        let (s2, r2) = bounded(1);
        slice_n.map_async(MapMode::Read, move |res| {
            let _ = s2.send(res);
        });
        device.poll(Maintain::Wait);
        let _ = r1.recv();
        let _ = r2.recv();
        let data_p = slice_p.get_mapped_range().to_vec();
        let data_n = slice_n.get_mapped_range().to_vec();
        positions.unmap();
        normals.unmap();
        (
            bytemuck::cast_slice(&data_p).to_vec(),
            bytemuck::cast_slice(&data_n).to_vec(),
        )
    }
}

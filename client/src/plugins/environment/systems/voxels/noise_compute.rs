use bevy::prelude::*;
use bevy_easy_compute::prelude::*;

#[repr(C)]
#[derive(ShaderType, Clone, Copy)]
pub struct NoiseParams {
    pub frequency: f32,
    pub amplitude: f32,
    pub width: u32,
    pub depth: u32,
}

#[derive(TypePath)]
struct NoiseShader;

impl ComputeShader for NoiseShader {
    fn shader() -> ShaderRef {
        "shaders/noise.wgsl".into()
    }
}

#[derive(Resource)]
pub struct NoiseWorker;

impl ComputeWorker for NoiseWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = NoiseParams {
            frequency: 0.1,
            amplitude: 1.0,
            width: 0,
            depth: 0,
        };

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_staging("heights", &[0.0_f32; 1])
            .add_pass::<NoiseShader>([1, 1, 1], &["params", "heights"])
            .one_shot()
            .build()
    }
}


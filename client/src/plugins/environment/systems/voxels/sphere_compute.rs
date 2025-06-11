use super::structure::{SparseVoxelOctree, Voxel};
use bevy::prelude::*;
use bevy_easy_compute::prelude::*;
use bytemuck::{Pod, Zeroable};

const SPHERE_RADIUS: u32 = 200;
const SPHERE_DIAMETER: u32 = SPHERE_RADIUS * 2 + 1;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Pod, Zeroable)]
pub struct SphereParams {
    pub radius: u32,
    pub diameter: u32,
}

#[derive(TypePath)]
struct SphereShader;

impl ComputeShader for SphereShader {
    fn shader() -> ShaderRef {
        "shaders/sphere.wgsl".into()
    }
}

#[derive(Resource)]
pub struct SphereWorker;

impl ComputeWorker for SphereWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let params = SphereParams {
            radius: SPHERE_RADIUS,
            diameter: SPHERE_DIAMETER,
        };
        let buffer = vec![0u32; (SPHERE_DIAMETER * SPHERE_DIAMETER * SPHERE_DIAMETER) as usize];

        AppComputeWorkerBuilder::new(world)
            .add_uniform("params", &params)
            .add_staging("voxels", &buffer)
            .add_pass::<SphereShader>(
                [
                    (SPHERE_DIAMETER + 7) / 8,
                    (SPHERE_DIAMETER + 7) / 8,
                    (SPHERE_DIAMETER + 7) / 8,
                ],
                &["params", "voxels"],
            )
            .synchronous()
            .one_shot()
            .build()
    }
}

#[derive(Resource, Default)]
pub struct SphereGenerated(pub bool);

pub fn execute_sphere_once(
    mut worker: ResMut<AppComputeWorker<SphereWorker>>,
    mut generated: ResMut<SphereGenerated>,
) {
    if generated.0 {
        return;
    }
    worker.execute();
    generated.0 = true;
}

pub fn apply_sphere_result(
    mut worker: ResMut<AppComputeWorker<SphereWorker>>,
    mut octree_q: Query<&mut SparseVoxelOctree>,
    mut generated: ResMut<SphereGenerated>,
) {
    if !generated.0 || !worker.ready() {
        return;
    }

    let voxels: Vec<u32> = worker.read_vec("voxels");
    let mut octree = octree_q.single_mut();
    let step = octree.get_spacing_at_depth(octree.max_depth);
    let radius = SPHERE_RADIUS as i32;
    let diameter = SPHERE_DIAMETER as i32;
    for x in 0..diameter {
        for y in 0..diameter {
            for z in 0..diameter {
                let idx = (z * diameter * diameter + y * diameter + x) as usize;
                if voxels[idx] != 0 {
                    let wx = (x - radius) as f32 * step;
                    let wy = (y - radius) as f32 * step;
                    let wz = (z - radius) as f32 * step;
                    octree.insert(
                        Vec3::new(wx, wy, wz),
                        Voxel {
                            color: Color::rgb(0.2, 0.8, 0.2),
                        },
                    );
                }
            }
        }
    }
    generated.0 = false;
}

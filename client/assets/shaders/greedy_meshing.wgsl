// Generates mesh quads for a voxel chunk using a simple greedy algorithm.
// Each invocation processes a slice of the chunk along one axis.
// Results are stored in a vertex/index buffer.

struct Params {
    origin: vec3<f32>,
    step: f32,
    axis: u32,
    dir: i32,
    slice: u32,
    n: vec3<f32>,
};

struct Vertex {
    pos: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

@group(0) @binding(0) var<storage, read> voxels: array<u32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(3) var<storage, read_write> indices: array<u32>;
@group(0) @binding(4) var<storage, read_write> counts: atomic<u32>;

const N: u32 = 16u;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // TODO: implement full greedy algorithm.
    // This shader currently only reserves space for CPU-driven meshing.
}

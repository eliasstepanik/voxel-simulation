struct Params {
    radius: u32,
    diameter: u32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read_write> voxels: array<u32>;

@compute @workgroup_size(8,8,8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.diameter || id.y >= params.diameter || id.z >= params.diameter) {
        return;
    }
    let r = f32(params.radius);
    let cx = f32(id.x) - r;
    let cy = f32(id.y) - r;
    let cz = f32(id.z) - r;
    let inside = select(0u, 1u, cx * cx + cy * cy + cz * cz <= r * r);
    let index = id.z * params.diameter * params.diameter + id.y * params.diameter + id.x;
    voxels[index] = inside;
}

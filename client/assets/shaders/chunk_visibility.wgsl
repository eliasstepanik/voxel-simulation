// Computes visible chunk keys based on camera centre and view radius.
// Input arrays must match in length and are processed per invocation.

struct Params {
    centre_radius: vec4<i32>;
    count: u32;
    _pad: u32;
};

@group(0) @binding(0) var<storage, read> occupied: array<vec3<i32>>;
@group(0) @binding(1) var<storage, read> spawned: array<u32>;
@group(0) @binding(2) var<storage, read_write> out_keys: array<vec3<i32>>;
@group(0) @binding(3) var<storage, read_write> out_count: atomic<u32>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if idx >= params.count { return; }
    let key = occupied[idx];
    if spawned[idx] != 0u { return; }
    let centre = params.centre_radius.xyz;
    let radius = params.centre_radius.w;
    let dx = key.x - centre.x;
    let dy = key.y - centre.y;
    let dz = key.z - centre.z;
    if dx*dx + dy*dy + dz*dz <= radius * radius {
        let i = atomicAdd(&out_count, 1u);
        out_keys[i] = key;
    }
}

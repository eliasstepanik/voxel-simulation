struct Params {
    centre: vec3<i32>;
    max_level: u32;
    range_step: f32;
    count: u32;
    _pad: u32;
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read> keys_in: array<vec3<i32>>;

@group(0) @binding(2)
var<storage, read_write> lod_out: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= params.count) { return; }
    let key = keys_in[idx];
    let dx = f32(key.x - params.centre.x);
    let dy = f32(key.y - params.centre.y);
    let dz = f32(key.z - params.centre.z);
    var level = floor(length(vec3<f32>(dx, dy, dz)) / params.range_step);
    if (level > f32(params.max_level)) { level = f32(params.max_level); }
    lod_out[idx] = u32(level);
}

struct Params {
    centre: vec3<i32>;
    radius: i32;
    count: u32;
    _pad: u32;
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read> keys_in: array<vec3<i32>>;

@group(0) @binding(2)
var<storage, read_write> results: array<vec4<i32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= params.count) { return; }
    let key = keys_in[idx];
    let dx = key.x - params.centre.x;
    let dy = key.y - params.centre.y;
    let dz = key.z - params.centre.z;
    var dist2 = dx * dx + dy * dy + dz * dz;
    if (abs(dx) > params.radius || abs(dy) > params.radius || abs(dz) > params.radius) {
        dist2 = -1;
    }
    results[idx] = vec4<i32>(key, dist2);
}

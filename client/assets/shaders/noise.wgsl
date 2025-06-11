struct Params {
    frequency: f32,
    amplitude: f32,
    width: u32,
    depth: u32,
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read_write> heights: array<f32>;

fn hash(p: vec2<i32>) -> f32 {
    let dot_val = f32(p.x * 1271 + p.y * 3117);
    return fract(sin(dot_val) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = vec2<i32>(floor(p));
    let f = fract(p);

    let a = hash(i);
    let b = hash(i + vec2<i32>(1,0));
    let c = hash(i + vec2<i32>(0,1));
    let d = hash(i + vec2<i32>(1,1));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.width || id.y >= params.depth) {
        return;
    }
    let index = id.y * params.width + id.x;
    let pos = vec2<f32>(f32(id.x), f32(id.y)) * params.frequency;
    let n = noise(pos);
    heights[index] = n * params.amplitude;
}

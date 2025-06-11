struct Params {
    origin: vec3<f32>;
    step: f32;
};

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read> voxels: array<u32>;

@group(0) @binding(2)
var<storage, read_write> positions: array<vec3<f32>>;

@group(0) @binding(3)
var<storage, read_write> normals: array<vec3<f32>>;

@group(0) @binding(4)
var<storage, read_write> uvs: array<vec2<f32>>;

@group(0) @binding(5)
var<storage, read_write> indices: array<u32>;

struct Counts {
    vertex: atomic<u32>;
    index: atomic<u32>;
};

@group(0) @binding(6)
var<storage, read_write> counts: Counts;

const N: u32 = 16u;

fn push_face(base: vec3<f32>, size: vec2<f32>, n: vec3<f32>, u: vec3<f32>, v: vec3<f32>) {
    let vi = atomicAdd(&counts.vertex, 4u);
    positions[vi + 0u] = base;
    positions[vi + 1u] = base + u * size.x;
    positions[vi + 2u] = base + u * size.x + v * size.y;
    positions[vi + 3u] = base + v * size.y;
    normals[vi + 0u] = n;
    normals[vi + 1u] = n;
    normals[vi + 2u] = n;
    normals[vi + 3u] = n;
    uvs[vi + 0u] = vec2<f32>(0.0, 1.0);
    uvs[vi + 1u] = vec2<f32>(1.0, 1.0);
    uvs[vi + 2u] = vec2<f32>(1.0, 0.0);
    uvs[vi + 3u] = vec2<f32>(0.0, 0.0);
    let ii = atomicAdd(&counts.index, 6u);
    if (n.x + n.y + n.z >= 0.0) {
        indices[ii + 0u] = vi;
        indices[ii + 1u] = vi + 1u;
        indices[ii + 2u] = vi + 2u;
        indices[ii + 3u] = vi + 2u;
        indices[ii + 4u] = vi + 3u;
        indices[ii + 5u] = vi;
    } else {
        indices[ii + 0u] = vi;
        indices[ii + 1u] = vi + 3u;
        indices[ii + 2u] = vi + 2u;
        indices[ii + 3u] = vi + 2u;
        indices[ii + 4u] = vi + 1u;
        indices[ii + 5u] = vi;
    }
}

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x > 0u) { return; }
    let s = params.step;
    for (var z: u32 = 0u; z < N; z = z + 1u) {
        for (var y: u32 = 0u; y < N; y = y + 1u) {
            for (var x: u32 = 0u; x < N; x = x + 1u) {
                let idx = z * N * N + y * N + x;
                if (voxels[idx] == 0u) { continue; }
                let base = params.origin + vec3<f32>(f32(x) * s, f32(y) * s, f32(z) * s);
                var filled: bool;
                // -X
                filled = false;
                if (x > 0u) { filled = voxels[idx - 1u] != 0u; }
                if (!filled) {
                    push_face(base, vec2<f32>(s, s), vec3<f32>(-1.0,0.0,0.0), vec3<f32>(0.0,0.0,s), vec3<f32>(0.0,s,0.0));
                }
                // +X
                filled = false;
                if (x + 1u < N) { filled = voxels[idx + 1u] != 0u; }
                if (!filled) {
                    let b = base + vec3<f32>(s,0.0,0.0);
                    push_face(b, vec2<f32>(s, s), vec3<f32>(1.0,0.0,0.0), vec3<f32>(0.0,s,0.0), vec3<f32>(0.0,0.0,s));
                }
                // -Y
                filled = false;
                if (y > 0u) { filled = voxels[idx - N] != 0u; }
                if (!filled) {
                    push_face(base, vec2<f32>(s, s), vec3<f32>(0.0,-1.0,0.0), vec3<f32>(s,0.0,0.0), vec3<f32>(0.0,0.0,s));
                }
                // +Y
                filled = false;
                if (y + 1u < N) { filled = voxels[idx + N] != 0u; }
                if (!filled) {
                    let b = base + vec3<f32>(0.0,s,0.0);
                    push_face(b, vec2<f32>(s, s), vec3<f32>(0.0,1.0,0.0), vec3<f32>(0.0,0.0,s), vec3<f32>(s,0.0,0.0));
                }
                // -Z
                filled = false;
                if (z > 0u) { filled = voxels[idx - N * N] != 0u; }
                if (!filled) {
                    push_face(base, vec2<f32>(s, s), vec3<f32>(0.0,0.0,-1.0), vec3<f32>(s,0.0,0.0), vec3<f32>(0.0,s,0.0));
                }
                // +Z
                filled = false;
                if (z + 1u < N) { filled = voxels[idx + N * N] != 0u; }
                if (!filled) {
                    let b = base + vec3<f32>(0.0,0.0,s);
                    push_face(b, vec2<f32>(s, s), vec3<f32>(0.0,0.0,1.0), vec3<f32>(s,0.0,0.0), vec3<f32>(0.0,s,0.0));
                }
            }
        }
    }
}

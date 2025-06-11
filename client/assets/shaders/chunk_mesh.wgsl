// Voxel occupancy input
@group(0) @binding(0)
var<storage, read> voxels: array<u32>;

// Outputs for the `count_faces` entry point
@group(0) @binding(1)
var<storage, read_write> face_mask: array<u32>;
@group(0) @binding(2)
var<storage, read_write> face_count: array<u32>;

// Buffers for the `generate_mesh` entry point
@group(0) @binding(3)
var<storage, read> prefix: array<u32>;
@group(0) @binding(4)
var<storage, read_write> positions: array<vec3<f32>>;
@group(0) @binding(5)
var<storage, read_write> normals: array<vec3<f32>>;

@group(0) @binding(7)
var<storage, read_write> face_total: array<u32>;

struct Params {
    origin: vec3<f32>,
    step: f32,
};
@group(0) @binding(6)
var<uniform> params: Params;

fn push_face(idx: ptr<function, u32>,
             v0: vec3<f32>, v1: vec3<f32>,
             v2: vec3<f32>, v3: vec3<f32>, n: vec3<f32>) {
    positions[*idx + 0u] = v0;
    normals[*idx + 0u] = n;
    positions[*idx + 1u] = v1;
    normals[*idx + 1u] = n;
    positions[*idx + 2u] = v2;
    normals[*idx + 2u] = n;
    positions[*idx + 3u] = v2;
    normals[*idx + 3u] = n;
    positions[*idx + 4u] = v3;
    normals[*idx + 4u] = n;
    positions[*idx + 5u] = v0;
    normals[*idx + 5u] = n;
    *idx = *idx + 6u;
}

// -----------------------------------------------------------------------------
// Prefix sum of face counts (sequential on GPU)
// -----------------------------------------------------------------------------
@compute @workgroup_size(1)
fn build_prefix(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x != 0u) {
        return;
    }
    var sum: u32 = 0u;
    for (var i: u32 = 0u; i < arrayLength(&face_count); i = i + 1u) {
        prefix[i] = sum;
        sum = sum + face_count[i];
    }
    face_total[0] = sum;
}

const CHUNK_SIZE: u32 = 16u;

fn index(x: u32, y: u32, z: u32) -> u32 {
    return x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;
}

// -----------------------------------------------------------------------------
// Count visible faces for each voxel
// -----------------------------------------------------------------------------
@compute @workgroup_size(64)
fn count_faces(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= arrayLength(&voxels)) {
        return;
    }
    let x = id % CHUNK_SIZE;
    let y = (id / CHUNK_SIZE) % CHUNK_SIZE;
    let z = id / (CHUNK_SIZE * CHUNK_SIZE);
    if (voxels[id] == 0u) {
        face_mask[id] = 0u;
        face_count[id] = 0u;
        return;
    }

    var mask: u32 = 0u;
    // -X
    if (x == 0u || voxels[index(x - 1u, y, z)] == 0u) {
        mask |= 1u << 0u;
    }
    // +X
    if (x + 1u >= CHUNK_SIZE || voxels[index(x + 1u, y, z)] == 0u) {
        mask |= 1u << 1u;
    }
    // -Y
    if (y == 0u || voxels[index(x, y - 1u, z)] == 0u) {
        mask |= 1u << 2u;
    }
    // +Y
    if (y + 1u >= CHUNK_SIZE || voxels[index(x, y + 1u, z)] == 0u) {
        mask |= 1u << 3u;
    }
    // -Z
    if (z == 0u || voxels[index(x, y, z - 1u)] == 0u) {
        mask |= 1u << 4u;
    }
    // +Z
    if (z + 1u >= CHUNK_SIZE || voxels[index(x, y, z + 1u)] == 0u) {
        mask |= 1u << 5u;
    }

    face_mask[id] = mask;
    face_count[id] = countOneBits(mask);
}

// -----------------------------------------------------------------------------
// Generate triangle vertices for each visible face
// -----------------------------------------------------------------------------
@compute @workgroup_size(64)
fn generate_mesh(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= arrayLength(&voxels)) {
        return;
    }

    var mask = face_mask[id];
    if (mask == 0u) {
        return;
    }

    let x = id % CHUNK_SIZE;
    let y = (id / CHUNK_SIZE) % CHUNK_SIZE;
    let z = id / (CHUNK_SIZE * CHUNK_SIZE);

    let base = params.origin + vec3<f32>(f32(x) * params.step,
                                         f32(y) * params.step,
                                         f32(z) * params.step);

    var out_idx = prefix[id] * 6u;
    let s = params.step;

    if ((mask & (1u << 0u)) != 0u) {
        push_face(&mut out_idx,
                  base,
                  base + vec3<f32>(0.0, s, 0.0),
                  base + vec3<f32>(0.0, s, s),
                  base + vec3<f32>(0.0, 0.0, s),
                  vec3<f32>(-1.0, 0.0, 0.0));
    }
    if ((mask & (1u << 1u)) != 0u) {
        let b = base + vec3<f32>(s, 0.0, 0.0);
        push_face(&mut out_idx,
                  b + vec3<f32>(0.0, 0.0, 0.0),
                  b + vec3<f32>(0.0, 0.0, s),
                  b + vec3<f32>(0.0, s, s),
                  b + vec3<f32>(0.0, s, 0.0),
                  vec3<f32>(1.0, 0.0, 0.0));
    }
    if ((mask & (1u << 2u)) != 0u) {
        push_face(&mut out_idx,
                  base,
                  base + vec3<f32>(0.0, 0.0, s),
                  base + vec3<f32>(s, 0.0, s),
                  base + vec3<f32>(s, 0.0, 0.0),
                  vec3<f32>(0.0, -1.0, 0.0));
    }
    if ((mask & (1u << 3u)) != 0u) {
        let b = base + vec3<f32>(0.0, s, 0.0);
        push_face(&mut out_idx,
                  b + vec3<f32>(0.0, 0.0, 0.0),
                  b + vec3<f32>(s, 0.0, 0.0),
                  b + vec3<f32>(s, 0.0, s),
                  b + vec3<f32>(0.0, 0.0, s),
                  vec3<f32>(0.0, 1.0, 0.0));
    }
    if ((mask & (1u << 4u)) != 0u) {
        push_face(&mut out_idx,
                  base,
                  base + vec3<f32>(s, 0.0, 0.0),
                  base + vec3<f32>(s, s, 0.0),
                  base + vec3<f32>(0.0, s, 0.0),
                  vec3<f32>(0.0, 0.0, -1.0));
    }
    if ((mask & (1u << 5u)) != 0u) {
        let b = base + vec3<f32>(0.0, 0.0, s);
        push_face(&mut out_idx,
                  b + vec3<f32>(0.0, s, 0.0),
                  b + vec3<f32>(s, s, 0.0),
                  b + vec3<f32>(s, 0.0, 0.0),
                  b + vec3<f32>(0.0, 0.0, 0.0),
                  vec3<f32>(0.0, 0.0, 1.0));
    }
}

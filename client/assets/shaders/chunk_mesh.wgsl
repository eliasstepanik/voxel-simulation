[[group(0), binding(0)]]
var<storage, read> voxels: array<u32>;
[[group(0), binding(1)]]
var<storage, read_write> face_mask: array<u32>;

const CHUNK_SIZE: u32 = 16u;

fn index(x: u32, y: u32, z: u32) -> u32 {
    return x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;
}

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
    let id = gid.x;
    if (id >= arrayLength(&voxels)) {
        return;
    }
    let x = id % CHUNK_SIZE;
    let y = (id / CHUNK_SIZE) % CHUNK_SIZE;
    let z = id / (CHUNK_SIZE * CHUNK_SIZE);
    if (voxels[id] == 0u) {
        face_mask[id] = 0u;
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
}

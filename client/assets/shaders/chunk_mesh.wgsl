[[group(0), binding(0)]]
var<storage, read> voxels: array<u32>;
[[group(0), binding(1)]]
var<storage, read_write> verts: array<u32>;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] gid: vec3<u32>) {
    // simple copy for now
    if (gid.x < arrayLength(&voxels)) {
        verts[gid.x] = voxels[gid.x];
    }
}

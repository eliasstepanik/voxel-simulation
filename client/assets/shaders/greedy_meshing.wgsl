// Generates mesh quads for a voxel chunk using a simple greedy algorithm.
// Each invocation processes a slice of the chunk along one axis.
// Results are stored in a vertex/index buffer.

struct Params {
    origin: vec3<f32>,
    step: f32,
    axis: u32,
    dir: i32,
    slice: u32,
    _pad: u32,
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

const MASK_LEN: u32 = N * N;

fn voxel_index(p: vec3<i32>) -> u32 {
    return u32(p.x) * N * N + u32(p.y) * N + u32(p.z);
}

fn voxel_filled(p: vec3<i32>) -> bool {
    return p.x >= 0 && p.x < i32(N) && p.y >= 0 && p.y < i32(N) && p.z >= 0 && p.z < i32(N) && voxels[voxel_index(p)] != 0u;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var mask: array<bool, MASK_LEN>;
    var visited: array<bool, MASK_LEN>;

    // Iterate over all axes and both face directions.
    for (var axis: u32 = 0u; axis < 3u; axis = axis + 1u) {
        for (var dir_idx: u32 = 0u; dir_idx < 2u; dir_idx = dir_idx + 1u) {
            let dir: i32 = select(-1, 1, dir_idx == 1u);

            for (var slice: u32 = 0u; slice < N; slice = slice + 1u) {
                // Build mask for this slice.
                for (var u: u32 = 0u; u < N; u = u + 1u) {
                    for (var v: u32 = 0u; v < N; v = v + 1u) {
                        var cell = vec3<i32>(0, 0, 0);
                        var neighbor = vec3<i32>(0, 0, 0);

                        if axis == 0u {
                            cell = vec3<i32>(i32(slice), i32(u), i32(v));
                            neighbor = cell + vec3<i32>(dir, 0, 0);
                        } else if axis == 1u {
                            cell = vec3<i32>(i32(v), i32(slice), i32(u));
                            neighbor = cell + vec3<i32>(0, dir, 0);
                        } else {
                            cell = vec3<i32>(i32(u), i32(v), i32(slice));
                            neighbor = cell + vec3<i32>(0, 0, dir);
                        }

                        let i = u * N + v;
                        mask[i] = voxel_filled(cell) && !voxel_filled(neighbor);
                        visited[i] = false;
                    }
                }

                // Greedy merge.
                for (var u0: u32 = 0u; u0 < N; u0 = u0 + 1u) {
                    for (var v0: u32 = 0u; v0 < N; v0 = v0 + 1u) {
                        let i0 = u0 * N + v0;
                        if !mask[i0] || visited[i0] {
                            continue;
                        }

                        var width: u32 = 1u;
                        loop {
                            if u0 + width >= N || !mask[u0 + width * N + v0] || visited[u0 + width * N + v0] {
                                break;
                            }
                            width = width + 1u;
                        }

                        var height: u32 = 1u;
                        loop {
                            if v0 + height >= N {
                                break;
                            }
                            var can_expand: bool = true;
                            for (var du: u32 = 0u; du < width; du = du + 1u) {
                                let idx = (u0 + du) * N + v0 + height;
                                if !mask[idx] || visited[idx] {
                                    can_expand = false;
                                    break;
                                }
                            }
                            if !can_expand {
                                break;
                            }
                            height = height + 1u;
                        }

                        for (var du: u32 = 0u; du < width; du = du + 1u) {
                            for (var dv: u32 = 0u; dv < height; dv = dv + 1u) {
                                visited[(u0 + du) * N + v0 + dv] = true;
                            }
                        }

                        // Compute base world-space position.
                        var base = params.origin;
                        if axis == 0u {
                            base = base + vec3<f32>(f32(slice) + select(0.0, 1.0, dir > 0), f32(u0), f32(v0)) * params.step;
                        } else if axis == 1u {
                            base = base + vec3<f32>(f32(v0), f32(slice) + select(0.0, 1.0, dir > 0), f32(u0)) * params.step;
                        } else {
                            base = base + vec3<f32>(f32(u0), f32(v0), f32(slice) + select(0.0, 1.0, dir > 0)) * params.step;
                        }

                        let size = vec2<f32>(f32(width) * params.step, f32(height) * params.step);

                        var normal = vec3<f32>(0.0, 0.0, 0.0);
                        var u_unit = vec3<f32>(0.0, 0.0, 0.0);
                        var v_unit = vec3<f32>(0.0, 0.0, 0.0);

                        if axis == 0u {
                            normal = vec3<f32>(f32(dir), 0.0, 0.0);
                            u_unit = vec3<f32>(0.0, 1.0, 0.0);
                            v_unit = vec3<f32>(0.0, 0.0, 1.0);
                        } else if axis == 1u {
                            normal = vec3<f32>(0.0, f32(dir), 0.0);
                            u_unit = vec3<f32>(0.0, 0.0, 1.0);
                            v_unit = vec3<f32>(1.0, 0.0, 0.0);
                        } else {
                            normal = vec3<f32>(0.0, 0.0, f32(dir));
                            u_unit = vec3<f32>(1.0, 0.0, 0.0);
                            v_unit = vec3<f32>(0.0, 1.0, 0.0);
                        }

                        let p0 = base;
                        let p1 = base + u_unit * size.x;
                        let p2 = base + u_unit * size.x + v_unit * size.y;
                        let p3 = base + v_unit * size.y;

                        let vi = atomicAdd(&counts[0], 4u);
                        vertices[vi] = Vertex(pos: p0, normal: normal, uv: vec2<f32>(0.0, 1.0));
                        vertices[vi + 1u] = Vertex(pos: p1, normal: normal, uv: vec2<f32>(1.0, 1.0));
                        vertices[vi + 2u] = Vertex(pos: p2, normal: normal, uv: vec2<f32>(1.0, 0.0));
                        vertices[vi + 3u] = Vertex(pos: p3, normal: normal, uv: vec2<f32>(0.0, 0.0));

                        let ii = atomicAdd(&counts[1], 6u);
                        if dir > 0 {
                            indices[ii] = vi;
                            indices[ii + 1u] = vi + 1u;
                            indices[ii + 2u] = vi + 2u;
                            indices[ii + 3u] = vi + 2u;
                            indices[ii + 4u] = vi + 3u;
                            indices[ii + 5u] = vi;
                        } else {
                            indices[ii] = vi;
                            indices[ii + 1u] = vi + 3u;
                            indices[ii + 2u] = vi + 2u;
                            indices[ii + 3u] = vi + 2u;
                            indices[ii + 4u] = vi + 1u;
                            indices[ii + 5u] = vi;
                        }
                    }
                }
            }
        }
    }
}

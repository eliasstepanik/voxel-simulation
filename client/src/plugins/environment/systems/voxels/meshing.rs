use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues, Mesh};
use crate::plugins::environment::systems::voxels::structure::*;

pub(crate) fn mesh_chunk(
    buffer: &[[[Option<Voxel>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    origin: Vec3,
    step:   f32,
    tree:   &SparseVoxelOctree,
) -> Mesh {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals   = Vec::<[f32; 3]>::new();
    let mut uvs       = Vec::<[f32; 2]>::new();
    let mut indices   = Vec::<u32>::new();

    // helper – safe test for a filled voxel
    let filled = |x: i32, y: i32, z: i32| -> bool {
        if (0..CHUNK_SIZE).contains(&x)
            && (0..CHUNK_SIZE).contains(&y)
            && (0..CHUNK_SIZE).contains(&z)
        {
            buffer[x as usize][y as usize][z as usize].is_some()
        } else {
            let world = origin + Vec3::new(x as f32 * step,
                                           y as f32 * step,
                                           z as f32 * step);
            tree.get_voxel_at_world_coords(world).is_some()
        }
    };

    // push a single quad
    let mut quad = |base: Vec3,
                    size: Vec2,
                    n:    Vec3,      // face normal (-1|+1 on one axis)
                    u:    Vec3,
                    v:    Vec3|
    {
        let i0 = positions.len() as u32;

        // 4 vertices -----------------------------------------------------------
        positions.extend_from_slice(&[
            (base).into(),
            (base + u * size.x).into(),
            (base + u * size.x + v * size.y).into(),
            (base + v * size.y).into(),
        ]);
        normals.extend_from_slice(&[[n.x, n.y, n.z]; 4]);
        uvs     .extend_from_slice(&[[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]);

        // indices -- flip for the negative-side faces -------------------------
        if n.x + n.y + n.z >= 0.0 {
            //  CCW   (front-face)
            indices.extend_from_slice(&[i0, i0 + 1, i0 + 2,  i0 + 2, i0 + 3, i0]);
        } else {
            //  CW → reverse two vertices so that the winding becomes CCW again
            indices.extend_from_slice(&[i0, i0 + 3, i0 + 2,  i0 + 2, i0 + 1, i0]);
        }
    };

    //-----------------------------------------------------------------------
    // Z–faces
    //-----------------------------------------------------------------------
    for z in 0..CHUNK_SIZE {                         //   -Z faces (normal −Z)
        let nz          = -1;
        let voxel_z     = z;
        let neighbour_z = voxel_z as i32 + nz;

        for y in 0..CHUNK_SIZE {
            let mut x = 0;
            while x < CHUNK_SIZE {
                if filled(x, y, voxel_z) && !filled(x, y, neighbour_z) {
                    // greedy run along +X
                    let run_start = x;
                    let mut run   = 1;
                    while x + run < CHUNK_SIZE
                        && filled(x + run, y, voxel_z)
                        && !filled(x + run, y, neighbour_z)
                    {
                        run += 1;
                    }

                    let face_z     = voxel_z as f32 * step + if nz == 1 { step } else { 0.0 };
                    let world_base = origin + Vec3::new(run_start as f32 * step, y as f32 * step, face_z);

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(0.0, 0.0, nz as f32),
                         Vec3::X,
                         Vec3::Y);

                    x += run;
                } else {
                    x += 1;
                }
            }
        }
    }
    
    // ------  2nd pass :  +Z faces  ---------------------------------------------
    for z in 0..CHUNK_SIZE {                         //   +Z faces (normal +Z)
        let nz          =  1;
        let voxel_z     = z;                         // this voxel
        let neighbour_z = voxel_z as i32 + nz;       // cell “in front of it”

        for y in 0..CHUNK_SIZE {
            let mut x = 0;
            while x < CHUNK_SIZE {
                if  filled(x, y, voxel_z) && !filled(x, y, neighbour_z)  {
                    let run_start = x;
                    let mut run   = 1;
                    while x + run < CHUNK_SIZE
                        && filled(x + run, y, voxel_z)
                        && !filled(x + run, y, neighbour_z)
                    { run += 1; }

                    let world_base = origin
                        + Vec3::new(run_start as f32 * step,
                                    y          as f32 * step,
                                    (voxel_z + 1) as f32 * step);   //  +1 !

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(0.0, 0.0, 1.0),                 //  +Z
                         Vec3::X,
                         Vec3::Y);

                    x += run;
                } else {
                    x += 1;
                }
            }
        }
    }


    // ────────────────────────────────────────────────────────────────────────────
    //  X faces  (-X pass … original code)
    // ────────────────────────────────────────────────────────────────────────────
    for x in 0..CHUNK_SIZE {                         //   -X faces (normal −X)
        let nx          = -1;
        let voxel_x     = x;
        let neighbour_x = voxel_x as i32 + nx;

        for z in 0..CHUNK_SIZE {
            let mut y = 0;
            while y < CHUNK_SIZE {
                if filled(voxel_x, y, z) && !filled(neighbour_x, y, z) {
                    let run_start = y;
                    let mut run   = 1;
                    while y + run < CHUNK_SIZE
                        && filled(voxel_x, y + run, z)
                        && !filled(neighbour_x, y + run, z)
                    { run += 1; }

                    // **fixed x-coordinate: add step when nx == +1**
                    let face_x = voxel_x as f32 * step + if nx == 1 { step } else { 0.0 };

                    let world_base = origin
                        + Vec3::new(face_x,
                                    run_start as f32 * step,
                                    z         as f32 * step);

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(nx as f32, 0.0, 0.0),
                         Vec3::Y,
                         Vec3::Z);

                    y += run;
                } else {
                    y += 1;
                }
            }
        }
    }

    // ------  2nd pass :  +X faces  ---------------------------------------------
    for x in 0..CHUNK_SIZE {                         //   +X faces (normal +X)
        let nx          =  1;
        let voxel_x     = x;
        let neighbour_x = voxel_x as i32 + nx;

        for z in 0..CHUNK_SIZE {
            let mut y = 0;
            while y < CHUNK_SIZE {
                if  filled(voxel_x, y, z) && !filled(neighbour_x, y, z)  {
                    let run_start = y;
                    let mut run   = 1;
                    while y + run < CHUNK_SIZE
                        && filled(voxel_x, y + run, z)
                        && !filled(neighbour_x, y + run, z)
                    { run += 1; }

                    let world_base = origin
                        + Vec3::new((voxel_x + 1) as f32 * step,    //  +1 !
                                    run_start as f32 * step,
                                    z         as f32 * step);

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(1.0, 0.0, 0.0),                 //  +X
                         Vec3::Y,
                         Vec3::Z);

                    y += run;
                } else {
                    y += 1;
                }
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────────────
    //  Y faces  (-Y pass … original code)
    // ────────────────────────────────────────────────────────────────────────────
    for y in 0..CHUNK_SIZE {                         //   -Y faces (normal −Y)
        let ny          = -1;
        let voxel_y     = y;
        let neighbour_y = voxel_y as i32 + ny;

        for x in 0..CHUNK_SIZE {
            let mut z = 0;
            while z < CHUNK_SIZE {
                if filled(x, voxel_y, z) && !filled(x, neighbour_y, z) {
                    let run_start = z;
                    let mut run   = 1;
                    while z + run < CHUNK_SIZE
                        && filled(x, voxel_y, z + run)
                        && !filled(x, neighbour_y, z + run)
                    { run += 1; }

                    // **fixed y-coordinate: add step when ny == +1**
                    let face_y = voxel_y as f32 * step + if ny == 1 { step } else { 0.0 };

                    let world_base = origin
                        + Vec3::new(x         as f32 * step,
                                    face_y,
                                    run_start as f32 * step);

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(0.0, ny as f32, 0.0),
                         Vec3::Z,
                         Vec3::X);

                    z += run;
                } else {
                    z += 1;
                }
            }
        }
    }
    // ------  2nd pass :  +Y faces  ---------------------------------------------
    for y in 0..CHUNK_SIZE {                         //   +Y faces (normal +Y)
        let ny          =  1;
        let voxel_y     = y;
        let neighbour_y = voxel_y as i32 + ny;

        for x in 0..CHUNK_SIZE {
            let mut z = 0;
            while z < CHUNK_SIZE {
                if  filled(x, voxel_y, z) && !filled(x, neighbour_y, z)  {
                    let run_start = z;
                    let mut run   = 1;
                    while z + run < CHUNK_SIZE
                        && filled(x, voxel_y, z + run)
                        && !filled(x, neighbour_y, z + run)
                    { run += 1; }

                    let world_base = origin
                        + Vec3::new(x         as f32 * step,
                                    (voxel_y + 1) as f32 * step,    //  +1 !
                                    run_start as f32 * step);

                    quad(world_base,
                         Vec2::new(run as f32 * step, step),
                         Vec3::new(0.0, 1.0, 0.0),                 //  +Y
                         Vec3::Z,
                         Vec3::X);

                    z += run;
                } else {
                    z += 1;
                }
            }
        }
    }

    //-----------------------------------------------------------------------
    // build final mesh
    //-----------------------------------------------------------------------
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,   VertexAttributeValues::Float32x3(normals));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     VertexAttributeValues::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
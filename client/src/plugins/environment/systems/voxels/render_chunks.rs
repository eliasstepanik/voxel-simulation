use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::gpu_meshing::GpuMesher;
use crate::plugins::environment::systems::voxels::meshing::mesh_chunk;
use crate::plugins::environment::systems::voxels::structure::*;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use big_space::prelude::GridCell;
use itertools::Itertools;
use std::collections::HashMap;

/// rebuilds meshes only for chunks flagged dirty by the octree
pub fn rebuild_dirty_chunks(
    mut commands: Commands,
    mut octrees: Query<&mut SparseVoxelOctree>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_q: Query<(
        Entity,
        &Chunk,
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &ChunkLod,
    )>,
    mut spawned: ResMut<SpawnedChunks>,
    root: Res<RootGrid>,
    mesher: Res<GpuMesher>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // map ChunkKey → (entity, mesh-handle, material-handle)
    let existing: HashMap<ChunkKey, (Entity, Handle<Mesh>, Handle<StandardMaterial>, u32)> =
        chunk_q
            .iter()
            .map(|(e, c, m, mat, lod)| (c.key, (e, m.0.clone(), mat.0.clone(), lod.0)))
            .collect();

    for mut tree in &mut octrees {
        if tree.dirty_chunks.is_empty() {
            continue;
        }

        //------------------------------------------------ collect voxel data
        let mut bufs = Vec::new();
        for key in tree.dirty_chunks.iter().copied() {
            let lod = existing.get(&key).map(|v| v.3).unwrap_or(0);
            let mut buf = [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

            let half = tree.size * 0.5;
            let step = tree.get_spacing_at_depth(tree.max_depth);
            let origin = Vec3::new(
                key.0 as f32 * CHUNK_SIZE as f32 * step - half,
                key.1 as f32 * CHUNK_SIZE as f32 * step - half,
                key.2 as f32 * CHUNK_SIZE as f32 * step - half,
            );

            let mult = 1 << lod;
            for gx in (0..CHUNK_SIZE).step_by(mult as usize) {
                for gy in (0..CHUNK_SIZE).step_by(mult as usize) {
                    for gz in (0..CHUNK_SIZE).step_by(mult as usize) {
                        let center = origin
                            + Vec3::new(
                                (gx + mult / 2) as f32 * step,
                                (gy + mult / 2) as f32 * step,
                                (gz + mult / 2) as f32 * step,
                            );
                        if let Some(v) = tree.get_voxel_at_world_coords(center) {
                            for lx in 0..mult {
                                for ly in 0..mult {
                                    for lz in 0..mult {
                                        let ix = gx + lx;
                                        let iy = gy + ly;
                                        let iz = gz + lz;
                                        if ix < CHUNK_SIZE && iy < CHUNK_SIZE && iz < CHUNK_SIZE {
                                            buf[ix as usize][iy as usize][iz as usize] = Some(*v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            bufs.push((key, buf, origin, step, lod));
        }

        //------------------------------------------------ create / update
        for (key, buf, origin, step, lod) in bufs {
            const N: usize = CHUNK_SIZE as usize;
            let index = |x: usize, y: usize, z: usize| -> usize { x + y * N + z * N * N };
            let mut occ = vec![0u32; N * N * N];
            for x in 0..N {
                for y in 0..N {
                    for z in 0..N {
                        if buf[x][y][z].is_some() {
                            occ[index(x, y, z)] = 1;
                        }
                    }
                }
            }
            let (mask, counts) = mesher.compute_face_mask(&render_device, &render_queue, &occ);
            let mut prefix = vec![0u32; counts.len()];
            let mut running = 0u32;
            for (i, c) in counts.iter().enumerate() {
                prefix[i] = running;
                running += *c;
            }
            let vertex_count = running as usize * 6;
            let (positions, normals) = mesher.generate_mesh(
                &render_device,
                &render_queue,
                &occ,
                &mask,
                &prefix,
                vertex_count,
                origin,
                step,
            );
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            if vertex_count == 0 {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
                mesh.insert_indices(Indices::U32(Vec::new()));
            } else {
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    VertexAttributeValues::Float32x3(positions),
                );
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    VertexAttributeValues::Float32x3(normals),
                );
                mesh.insert_indices(Indices::U32((0..vertex_count as u32).collect()));
            }

            if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                if vertex_count > 0 {
                    if let Some(mesh_asset) = meshes.get_mut(&mesh_h) {
                        *mesh_asset = mesh;
                    }
                    spawned.0.insert(key, ent);
                } else {
                    meshes.remove(&mesh_h);
                    commands.entity(ent).despawn_recursive();
                    spawned.0.remove(&key);
                }
            } else if vertex_count > 0 {
                let mesh_h = meshes.add(mesh);
                let mat_h = materials.add(StandardMaterial::default());

                commands.entity(root.0).with_children(|p| {
                    let e = p
                        .spawn((
                            Mesh3d::from(mesh_h.clone()),
                            MeshMaterial3d(mat_h.clone()),
                            Transform::default(),
                            GridCell::<i64>::ZERO,
                            Chunk {
                                key,
                                voxels: Vec::new(),
                                dirty: false,
                            },
                            ChunkLod(lod),
                            /*Wireframe,*/
                        ))
                        .id();
                    spawned.0.insert(key, e);
                });
            }
        }

        tree.clear_dirty_flags();
    }
}

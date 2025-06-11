use std::collections::HashMap;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use big_space::prelude::GridCell;
use itertools::Itertools;
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::meshing::{mesh_chunk, mesh_chunk_with_mask};
use crate::plugins::environment::systems::voxels::gpu_meshing::GpuMesher;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use crate::plugins::environment::systems::voxels::structure::*;

/// rebuilds meshes only for chunks flagged dirty by the octree
pub fn rebuild_dirty_chunks(
    mut commands : Commands,
    mut octrees  : Query<&mut SparseVoxelOctree>,
    mut meshes   : ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_q      : Query<(Entity,
                          &Chunk,
                          &Mesh3d,
                          &MeshMaterial3d<StandardMaterial>,
                          &ChunkLod)>,
    mut spawned  : ResMut<SpawnedChunks>,
    root         : Res<RootGrid>,
    mesher       : Res<GpuMesher>,
    render_device: Res<RenderDevice>,
    render_queue : Res<RenderQueue>,
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
            let mut buf =
                [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

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
            let (mask, _counts) = mesher.compute_face_mask(&render_device, &render_queue, &occ);
            if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                // update mesh in-place; keeps old asset id
                match mesh_chunk_with_mask(&buf, &mask, origin, step, &tree) {
                    Some(new_mesh) => {
                        if let Some(mesh) = meshes.get_mut(&mesh_h) {
                            *mesh = new_mesh;
                        }
                        spawned.0.insert(key, ent);
                    }
                    None => {
                        meshes.remove(&mesh_h);
                        commands.entity(ent).despawn_recursive();
                        spawned.0.remove(&key);
                    }
                }
            } else if let Some(mesh) = mesh_chunk_with_mask(&buf, &mask, origin, step, &tree) {
                // spawn brand-new chunk only if mesh has faces
                let mesh_h = meshes.add(mesh);
                let mat_h  = materials.add(StandardMaterial::default());

                commands.entity(root.0).with_children(|p| {
                    let e = p
                        .spawn((
                            Mesh3d::from(mesh_h.clone()),
                            MeshMaterial3d(mat_h.clone()),
                            Transform::default(),
                            GridCell::<i64>::ZERO,
                            Chunk { key, voxels: Vec::new(), dirty: false },
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
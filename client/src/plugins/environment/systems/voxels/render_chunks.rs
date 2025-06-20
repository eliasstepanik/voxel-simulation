use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::atlas::VoxelTextureAtlas;
use crate::plugins::environment::systems::voxels::meshing::mesh_chunk;
use crate::plugins::environment::systems::voxels::structure::*;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use big_space::prelude::GridCell;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt::format;

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
    mut pool: ResMut<MeshBufferPool>,
    root: Res<RootGrid>,
    atlas: Res<VoxelTextureAtlas>,
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
        let dirty_keys: Vec<_> = tree.dirty_chunks.iter().copied().collect();

        for key in dirty_keys {
            let lod = existing.get(&key).map(|v| v.3).unwrap_or(0);
            let mut buf = [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

            let half = tree.size * 0.5;
            let step = tree.get_spacing_at_depth(tree.max_depth);
            let origin = Vec3::new(
                tree.center.x - half + key.0 as f32 * CHUNK_SIZE as f32 * step,
                tree.center.y - half + key.1 as f32 * CHUNK_SIZE as f32 * step,
                tree.center.z - half + key.2 as f32 * CHUNK_SIZE as f32 * step,
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

            if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                match mesh_chunk(&buf, origin, step, &tree, &mut pool, &atlas) {
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
            } else if let Some(mesh) = mesh_chunk(&buf, origin, step, &tree, &mut pool, &atlas) {
                let mesh_h = meshes.add(mesh);
                let mat_h = materials.add(StandardMaterial {
                    base_color_texture: Some(atlas.handle.clone()),
                    ..Default::default()
                });

                commands.entity(root.0).with_children(|p| {
                    let e = p
                        .spawn((
                            Mesh3d::from(mesh_h.clone()),
                            MeshMaterial3d(mat_h.clone()),
                            Transform::default(),
                            GridCell::ZERO,
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

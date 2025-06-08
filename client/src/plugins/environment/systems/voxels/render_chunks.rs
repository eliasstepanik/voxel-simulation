use std::collections::HashMap;
use std::fmt::format;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use big_space::prelude::GridCell;
use itertools::Itertools;
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::meshing::mesh_chunk;
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
                          &MeshMaterial3d<StandardMaterial>)>,
    mut spawned  : ResMut<SpawnedChunks>,
    root         : Res<RootGrid>,
) {
    // map ChunkKey → (entity, mesh-handle, material-handle)
    let existing: HashMap<ChunkKey, (Entity, Handle<Mesh>, Handle<StandardMaterial>)> =
        chunk_q
            .iter()
            .map(|(e, c, m, mat)| (c.key, (e, m.0.clone(), mat.0.clone())))
            .collect();

    for mut tree in &mut octrees {
        if tree.dirty_chunks.is_empty() {
            continue;
        }

        //------------------------------------------------ collect voxel data
        let mut bufs = Vec::new();
        for key in tree.dirty_chunks.iter().copied() {
            let mut buf =
                [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

            let half = tree.size * 0.5;
            let step = tree.get_spacing_at_depth(tree.max_depth);
            let origin = Vec3::new(
                key.0 as f32 * CHUNK_SIZE as f32 * step - half,
                key.1 as f32 * CHUNK_SIZE as f32 * step - half,
                key.2 as f32 * CHUNK_SIZE as f32 * step - half,
            );

            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let world = origin
                            + Vec3::new(lx as f32 * step, ly as f32 * step, lz as f32 * step);
                        if let Some(v) = tree.get_voxel_at_world_coords(world) {
                            buf[lx as usize][ly as usize][lz as usize] = Some(*v);
                        }
                    }
                }
            }

            bufs.push((key, buf, origin, step));
        }

        //------------------------------------------------ create / update
        for (key, buf, origin, step) in bufs {
            if let Some((ent, mesh_h, _mat_h)) = existing.get(&key).cloned() {
                // update mesh in-place; keeps old asset id
                if let Some(mesh) = meshes.get_mut(&mesh_h) {
                    *mesh = mesh_chunk(&buf, origin, step, &tree);
                }
                spawned.0.insert(key, ent);
            } else {
                // spawn brand-new chunk
                let mesh_h = meshes.add(mesh_chunk(&buf, origin, step, &tree));
                let mat_h  = materials.add(StandardMaterial::default());

                commands.entity(root.0).with_children(|p| {
                    let e = p
                        .spawn((
                            Mesh3d::from(mesh_h.clone()),
                            MeshMaterial3d(mat_h.clone()),
                            Transform::default(),
                            GridCell::<i64>::ZERO,
                            Chunk { key, voxels: Vec::new(), dirty: false },
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
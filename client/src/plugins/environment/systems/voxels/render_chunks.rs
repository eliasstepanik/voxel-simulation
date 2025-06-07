use std::collections::HashMap;
use std::fmt::format;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use big_space::prelude::GridCell;
use itertools::Itertools;
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::chunk::Chunk;
use crate::plugins::environment::systems::voxels::meshing::mesh_chunk;
use crate::plugins::environment::systems::voxels::structure::*;
/// rebuilds meshes only for chunks flagged dirty by the octree
pub fn rebuild_dirty_chunks(
    mut commands:   Commands,
    mut octrees:    Query<(Entity, &mut SparseVoxelOctree)>,
    mut meshes:     ResMut<Assets<Mesh>>,
    mut materials:  ResMut<Assets<StandardMaterial>>,
    chunk_q:        Query<(Entity, &Chunk)>,
    root:           Res<RootGrid>,
) {
    // map ChunkKey → entity
    
    let existing: HashMap<ChunkKey, Entity> =
        chunk_q.iter().map(|(e, c)| (c.key, e)).collect();

    for (_tree_ent, mut tree) in &mut octrees {
        if tree.dirty_chunks.is_empty() {
            continue;
        }

        // gather voxel data for every dirty chunk
        let mut chunk_voxel_bufs: Vec<(
            ChunkKey,
            [[[Option<Voxel>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
            Vec3, // chunk origin
            f32,  // voxel step
        )> = Vec::new();

        for key in tree.dirty_chunks.iter().copied() {
            let mut buf =
                [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

            let half = tree.size * 0.5;
            let step = tree.get_spacing_at_depth(tree.max_depth);
            let start = Vec3::new(
                key.0 as f32 * CHUNK_SIZE as f32 * step - half,
                key.1 as f32 * CHUNK_SIZE as f32 * step - half,
                key.2 as f32 * CHUNK_SIZE as f32 * step - half,
            );

            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let world = start
                            + Vec3::new(lx as f32 * step, ly as f32 * step, lz as f32 * step);
                        if let Some(v) = tree.get_voxel_at_world_coords(world) {
                            buf[lx as usize][ly as usize][lz as usize] = Some(*v);
                        }
                    }
                }
            }

            chunk_voxel_bufs.push((key, buf, start, step));
        }

        // build / replace meshes
        for (key, buf, origin, step) in chunk_voxel_bufs {
            let mesh_handle =
                meshes.add(mesh_chunk(&buf, origin, step, &tree));
            let mesh_3d     = Mesh3d::from(mesh_handle);
            let material    = MeshMaterial3d::<StandardMaterial>::default();

            if let Some(&ent) = existing.get(&key) {
                commands.entity(ent).insert(mesh_3d);
            } else {
                commands.entity(root.0).with_children(|p| {
                    p.spawn((
                        mesh_3d,
                        material,
                        Transform::default(),
                        GridCell::<i64>::ZERO,
                        Chunk { key, voxels: Vec::new(), dirty: false },
                    ));
                });
            }
        }

        tree.clear_dirty_flags();
    }
}
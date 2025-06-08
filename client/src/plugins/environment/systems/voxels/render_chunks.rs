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

/// distance from the camera at which LOD starts degrading
const LOD_START_DISTANCE: f32 = 30.0;
/// controls how aggressively detail decreases with distance (<1.0 = slower)
const LOD_DISTANCE_SCALE: f32 = 0.5;

fn lod_depth(distance: f32, max_depth: u32) -> u32 {
    let scaled = (distance / LOD_START_DISTANCE).max(1.0).log2();
    let drop = (scaled / LOD_DISTANCE_SCALE).floor() as i32;
    max_depth.saturating_sub(drop as u32)
}

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
    cam_q        : Query<&GlobalTransform, With<Camera>>,
) {
    // map ChunkKey → (entity, mesh-handle, material-handle)
    let existing: HashMap<ChunkKey, (Entity, Handle<Mesh>, Handle<StandardMaterial>)> =
        chunk_q
            .iter()
            .map(|(e, c, m, mat)| (c.key, (e, m.0.clone(), mat.0.clone())))
            .collect();

    let cam_pos = cam_q.single().translation();

    for mut tree in &mut octrees {
        if tree.dirty_chunks.is_empty() {
            continue;
        }

        //------------------------------------------------ collect voxel data
        let mut bufs = Vec::new();
        let step_max = tree.get_spacing_at_depth(tree.max_depth);

        for key in tree.dirty_chunks.iter().copied() {
            let mut buf =
                [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

            let half = tree.size * 0.5;
            let origin = Vec3::new(
                key.0 as f32 * CHUNK_SIZE as f32 * step_max - half,
                key.1 as f32 * CHUNK_SIZE as f32 * step_max - half,
                key.2 as f32 * CHUNK_SIZE as f32 * step_max - half,
            );

            let chunk_center = origin + Vec3::splat(step_max * CHUNK_SIZE as f32 * 0.5);
            let distance = cam_pos.distance(chunk_center);
            let depth = lod_depth(distance, tree.max_depth);
            let step = tree.get_spacing_at_depth(depth);

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
use std::collections::HashMap;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::{
    Mesh, PrimitiveTopology, Indices, VertexAttributeValues, RenderAssetUsages,
};
use big_space::prelude::GridCell;
use crate::plugins::big_space::big_space_plugin::RootGrid;
use crate::plugins::environment::systems::voxels::chunk_mesh_compute::{
    ChunkMeshWorker, MeshParams, MeshCounts,
};
use bevy_easy_compute::prelude::AppComputeWorker;
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
    mut worker   : ResMut<AppComputeWorker<ChunkMeshWorker>>,
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
            let voxels: Vec<u32> = buf
                .iter()
                .flat_map(|p| p.iter().flat_map(|r| r.iter().map(|v| if v.is_some() { 1u32 } else { 0u32 })))
                .collect();
            let params = MeshParams { origin: [origin.x, origin.y, origin.z], step };
            worker.write("params", &params);
            worker.write_slice("voxels", &voxels);
            worker.write("counts", &MeshCounts::default());
            worker.execute();
            let counts: MeshCounts = worker.read("counts");
            let mut positions: Vec<[f32; 3]> = worker.read_vec("positions");
            let mut normals: Vec<[f32; 3]> = worker.read_vec("normals");
            let mut uvs: Vec<[f32; 2]> = worker.read_vec("uvs");
            let mut indices: Vec<u32> = worker.read_vec("indices");
            positions.truncate(counts.vertex_count as usize);
            normals.truncate(counts.vertex_count as usize);
            uvs.truncate(counts.vertex_count as usize);
            indices.truncate(counts.index_count as usize);
            let mesh_opt = if counts.index_count > 0 {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
                mesh.insert_indices(Indices::U32(indices));
                Some(mesh)
            } else { None };

            if let Some((ent, mesh_h, _mat_h, _)) = existing.get(&key).cloned() {
                // update mesh in-place; keeps old asset id
                match mesh_opt {
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
            } else if let Some(mesh) = mesh_opt {
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
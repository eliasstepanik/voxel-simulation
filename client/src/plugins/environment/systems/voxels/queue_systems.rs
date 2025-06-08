use bevy::prelude::*;
use crate::plugins::environment::systems::voxels::helper::world_to_chunk;
use crate::plugins::environment::systems::voxels::structure::*;



/// enqueue chunks that *should* be visible but are not yet spawned
/// enqueue chunks that *should* be visible but are not yet spawned
pub fn enqueue_visible_chunks(
    mut queue      : ResMut<ChunkQueue>,
    spawned        : Res<SpawnedChunks>,
    cfg            : Res<ChunkCullingCfg>,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
) {
    let tree     = tree_q.single();
    let cam_pos  = cam_q.single().translation();
    let centre   = world_to_chunk(tree, cam_pos);
    let r        = cfg.view_distance_chunks;

    // ------------------------------------------------------------------
    // 1. gather every *new* candidate chunk together with its distance²
    // ------------------------------------------------------------------
    let mut candidates: Vec<(i32 /*dist²*/, ChunkKey)> = Vec::new();

    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                let key = ChunkKey(centre.0 + dx, centre.1 + dy, centre.2 + dz);

                if spawned.0.contains_key(&key) { continue; }   // already spawned
                if queue.0.contains(&key)       { continue; }   // already queued
                if !tree.chunk_has_any_voxel(key) { continue; } // empty air

                let dist2 = dx*dx + dy*dy + dz*dz;              // squared distance
                candidates.push((dist2, key));
            }
        }
    }

    // ------------------------------------------------------------------
    // 2. sort by distance so nearest chunks enter the queue first
    // ------------------------------------------------------------------
    candidates.sort_by_key(|&(d2, _)| d2);

    // push into FIFO queue in that order
    for (_, key) in candidates {
        queue.0.push_back(key);
    }
}

/// move a limited number of keys from the queue into the octree’s dirty set
pub fn process_chunk_queue(
    mut queue   : ResMut<ChunkQueue>,
    budget      : Res<ChunkBudget>,
    mut tree_q  : Query<&mut SparseVoxelOctree>,
) {
    let mut tree = tree_q.single_mut();
    for _ in 0..budget.per_frame {
        if let Some(key) = queue.0.pop_front() {
            tree.dirty_chunks.insert(key);
        } else { break; }
    }
}

/// enqueue far away chunks for low resolution rendering when the main queue is
/// not filling the entire budget
pub fn enqueue_lod_chunks(
    mut queue      : ResMut<LodChunkQueue>,
    spawned        : Res<SpawnedChunks>,
    cfg            : Res<ChunkCullingCfg>,
    cam_q          : Query<&GlobalTransform, With<Camera>>,
    tree_q         : Query<&SparseVoxelOctree>,
    main_queue     : Res<ChunkQueue>,
    mut search     : ResMut<LodSearchState>,
    budget         : Res<ChunkBudget>,
) {
    // only spend spare budget on far chunks
    if main_queue.0.len() >= budget.per_frame { return; }
    let capacity = budget.per_frame - main_queue.0.len();

    let tree    = tree_q.single();
    let cam_pos = cam_q.single().translation();
    let centre  = world_to_chunk(tree, cam_pos);

    if search.last_center.map_or(true, |c| c != centre) {
        search.last_center = Some(centre);
        search.index = 0;
        let mut tmp: Vec<_> = queue.0.drain(..).collect();
        tmp.sort_by_key(|key| {
            let ChunkKey(x, y, z) = *key;
            let dx = x - centre.0;
            let dy = y - centre.1;
            let dz = z - centre.2;
            dx * dx + dy * dy + dz * dz
        });
        queue.0.extend(tmp);
    }

    let checks_per_frame = capacity.max(1) * 2;
    if search.offsets.is_empty() {
        search.rebuild(cfg.as_ref());
    }
    let total = search.offsets.len();
    for _ in 0..checks_per_frame {
        if total == 0 { break; }
        let off   = search.offsets[search.index];
        search.index = (search.index + 1) % total;
        let key   = ChunkKey(centre.0 + off.x, centre.1 + off.y, centre.2 + off.z);
        if spawned.0.contains_key(&key) { continue; }
        if queue.0.contains(&key) { continue; }
        if !tree.chunk_has_any_voxel(key) { continue; }
        queue.0.push_back(key);
        if queue.0.len() >= capacity { break; }
    }
}

/// spawn low resolution chunk meshes directly from the LOD queue
pub fn process_lod_chunk_queue(
    mut commands   : Commands,
    mut queue      : ResMut<LodChunkQueue>,
    budget         : Res<ChunkBudget>,
    tree_q         : Query<&SparseVoxelOctree>,
    mut meshes     : ResMut<Assets<Mesh>>,
    mut materials  : ResMut<Assets<StandardMaterial>>,
    mut spawned    : ResMut<SpawnedChunks>,
    root           : Res<RootGrid>,
    cfg            : Res<ChunkCullingCfg>,
) {
    let tree = tree_q.single();
    let step = tree.get_spacing_at_depth(tree.max_depth.saturating_sub(cfg.lod_level));
    let half = tree.size * 0.5;

    for _ in 0..budget.per_frame {
        if let Some(key) = queue.0.pop_front() {
            if spawned.0.contains_key(&key) { continue; }

            let mut buf = [[[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
            let origin = Vec3::new(
                key.0 as f32 * CHUNK_SIZE as f32 * step - half,
                key.1 as f32 * CHUNK_SIZE as f32 * step - half,
                key.2 as f32 * CHUNK_SIZE as f32 * step - half,
            );

            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let world = origin + Vec3::new(lx as f32 * step, ly as f32 * step, lz as f32 * step);
                        if let Some(v) = tree.get_voxel_at_world_coords(world) {
                            buf[lx as usize][ly as usize][lz as usize] = Some(*v);
                        }
                    }
                }
            }

            let mesh_handle = meshes.add(mesh_chunk(&buf, origin, step, tree));
            let mesh_3d     = Mesh3d::from(mesh_handle);
            let material    = MeshMaterial3d::<StandardMaterial>::default();

            commands.entity(root.0).with_children(|p| {
                let e = p.spawn((
                    mesh_3d,
                    material,
                    Transform::default(),
                    GridCell::<i64>::ZERO,
                    Chunk { key, voxels: Vec::new(), dirty: false },
                    LodLevel(cfg.lod_level),
                )).id();
                spawned.0.insert(key, e);
            });
        } else { break; }
    }
}
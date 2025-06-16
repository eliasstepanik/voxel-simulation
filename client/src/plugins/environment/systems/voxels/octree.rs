use crate::plugins::environment::systems::voxels::helper::chunk_key_from_world;
use crate::plugins::environment::systems::voxels::structure::{
    AABB, CHUNK_SIZE, ChunkKey, DirtyVoxel, NEIGHBOR_OFFSETS, OctreeNode, Ray, SparseVoxelOctree,
    Voxel,
};
use bevy::asset::Assets;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bincode;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

impl SparseVoxelOctree {
    /// Creates a new octree with the specified max depth, size, and wireframe visibility.
    pub fn new(
        max_depth: u32,
        size: f32,
        show_wireframe: bool,
        show_world_grid: bool,
        show_chunks: bool,
    ) -> Self {
        Self {
            root: OctreeNode::new(),
            max_depth,
            size,
            center: Vec3::ZERO,
            show_wireframe,
            show_world_grid,
            dirty: Vec::new(),
            dirty_chunks: Default::default(),
            occupied_chunks: Default::default(),
        }
    }
    pub fn insert(&mut self, position: Vec3, voxel: Voxel) {
        // Align to the center of the voxel at max_depth
        let mut aligned = self.normalize_to_voxel_at_depth(position, self.max_depth);
        let mut world_center = self.denormalize_voxel_center(aligned);

        // Expand as needed using the denormalized position.
        while !self.contains(world_center.x, world_center.y, world_center.z) {
            self.expand_root(world_center.x, world_center.y, world_center.z);
            // Recompute aligned and world_center after expansion.
            aligned = self.normalize_to_voxel_at_depth(position, self.max_depth);
            world_center = self.denormalize_voxel_center(aligned);
        }

        let dirty_voxel = DirtyVoxel { position: aligned };

        self.dirty.push(dirty_voxel);
        let key = chunk_key_from_world(self, position);
        self.dirty_chunks.insert(key);
        self.mark_neighbor_chunks_dirty(position);
        self.occupied_chunks.insert(key);

        // iterative insertion to avoid deep recursion when the tree grows
        let mut node = &mut self.root;
        let mut pos = aligned;
        let mut depth = self.max_depth;
        let epsilon = 1e-6;
        while depth > 0 {
            if node.children.is_none() {
                node.children = Some(Box::new(core::array::from_fn(|_| OctreeNode::new())));
                node.is_leaf = false;
            }
            let index = ((pos.x >= 0.5 - epsilon) as usize)
                + ((pos.y >= 0.5 - epsilon) as usize * 2)
                + ((pos.z >= 0.5 - epsilon) as usize * 4);
            let adjust = |coord: f32| {
                if coord >= 0.5 - epsilon {
                    (coord - 0.5) * 2.0
                } else {
                    coord * 2.0
                }
            };
            pos = Vec3::new(adjust(pos.x), adjust(pos.y), adjust(pos.z));
            node = node.children.as_mut().unwrap().get_mut(index).unwrap();
            depth -= 1;
        }
        node.voxel = Some(voxel);
        node.is_leaf = true;
    }


    pub fn remove(&mut self, position: Vec3) {
        let aligned = self.normalize_to_voxel_at_depth(position, self.max_depth);

        self.dirty.push(DirtyVoxel { position: aligned });

        // mark the chunk
        let key = chunk_key_from_world(self, position);
        self.dirty_chunks.insert(key);
        self.mark_neighbor_chunks_dirty(position);

        Self::remove_recursive(
            &mut self.root,
            aligned.x,
            aligned.y,
            aligned.z,
            self.max_depth,
        );

        if !self.chunk_has_any_voxel(key) {
            self.occupied_chunks.remove(&key);
        }
    }

    pub fn clear_dirty_flags(&mut self) {
        self.dirty.clear();
        self.dirty_chunks.clear();
    }

    fn mark_neighbor_chunks_dirty(&mut self, position: Vec3) {
        let key = chunk_key_from_world(self, position);
        let step = self.get_spacing_at_depth(self.max_depth);
        let half = self.size * 0.5;

        let gx = ((position.x - self.center.x + half) / step).floor() as i32;
        let gy = ((position.y - self.center.y + half) / step).floor() as i32;
        let gz = ((position.z - self.center.z + half) / step).floor() as i32;

        let lx = gx - key.0 * CHUNK_SIZE;
        let ly = gy - key.1 * CHUNK_SIZE;
        let lz = gz - key.2 * CHUNK_SIZE;

        let mut neighbors = [
            (lx == 0, ChunkKey(key.0 - 1, key.1, key.2)),
            (lx == CHUNK_SIZE - 1, ChunkKey(key.0 + 1, key.1, key.2)),
            (ly == 0, ChunkKey(key.0, key.1 - 1, key.2)),
            (ly == CHUNK_SIZE - 1, ChunkKey(key.0, key.1 + 1, key.2)),
            (lz == 0, ChunkKey(key.0, key.1, key.2 - 1)),
            (lz == CHUNK_SIZE - 1, ChunkKey(key.0, key.1, key.2 + 1)),
        ];

        for (cond, n) in neighbors.iter() {
            if *cond && self.occupied_chunks.contains(n) {
                self.dirty_chunks.insert(*n);
            }
        }
    }

    /// Mark all six neighbor chunks of the given key as dirty if they exist.
    pub fn mark_neighbors_dirty_from_key(&mut self, key: ChunkKey) {
        let offsets = [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ];
        for (dx, dy, dz) in offsets {
            let neighbor = ChunkKey(key.0 + dx, key.1 + dy, key.2 + dz);
            if self.occupied_chunks.contains(&neighbor) {
                self.dirty_chunks.insert(neighbor);
            }
        }
    }

    /// Insert a sphere of voxels with the given radius (in voxels) and center.
    pub fn insert_sphere(&mut self, center: Vec3, radius: i32, voxel: Voxel) {
        let step = self.get_spacing_at_depth(self.max_depth);
        let r2 = radius * radius;

        for x in -radius..=radius {
            let dx2 = x * x;
            for y in -radius..=radius {
                let dy2 = y * y;
                for z in -radius..=radius {
                    let dz2 = z * z;
                    if dx2 + dy2 + dz2 <= r2 {
                        let pos = Vec3::new(
                            center.x + x as f32 * step,
                            center.y + y as f32 * step,
                            center.z + z as f32 * step,
                        );
                        self.insert(pos, voxel);
                    }
                }
            }
        }
    }

    /// Remove all voxels inside a sphere with the given radius (in voxels).
    pub fn remove_sphere(&mut self, center: Vec3, radius: i32) {
        let step = self.get_spacing_at_depth(self.max_depth);
        let r2 = radius * radius;

        for x in -radius..=radius {
            let dx2 = x * x;
            for y in -radius..=radius {
                let dy2 = y * y;
                for z in -radius..=radius {
                    let dz2 = z * z;
                    if dx2 + dy2 + dz2 <= r2 {
                        let pos = Vec3::new(
                            center.x + x as f32 * step,
                            center.y + y as f32 * step,
                            center.z + z as f32 * step,
                        );
                        self.remove(pos);
                    }
                }
            }
        }
    }

    fn remove_recursive(node: &mut OctreeNode, mut x: f32, mut y: f32, mut z: f32, mut depth: u32) -> bool {
        let epsilon = 1e-6;
        let mut path: Vec<(*mut OctreeNode, usize)> = Vec::new();
        let mut current: *mut OctreeNode = node;
        // descend to the target node, recording the path
        while depth > 0 {
            unsafe {
                if (*current).children.is_none() {
                    return false;
                }
                let index = ((x >= 0.5 - epsilon) as usize)
                    + ((y >= 0.5 - epsilon) as usize * 2)
                    + ((z >= 0.5 - epsilon) as usize * 4);
                path.push((current, index));
                let adjust = |coord: f32| {
                    if coord >= 0.5 - epsilon {
                        (coord - 0.5) * 2.0
                    } else {
                        coord * 2.0
                    }
                };
                let children = (*current).children.as_mut().unwrap();
                current = &mut children[index];
                x = adjust(x);
                y = adjust(y);
                z = adjust(z);
                depth -= 1;
            }
        }

        let mut removed = false;
        unsafe {
            if (*current).voxel.is_some() {
                (*current).voxel = None;
                (*current).is_leaf = false;
                removed = true;
            }
        }

        if !removed {
            return false;
        }

        // walk back up pruning empty children
        while let Some((ptr, idx)) = path.pop() {
            unsafe {
                let parent = &mut *ptr;
                let child = &mut parent.children.as_mut().unwrap()[idx];
                if child.is_empty() {
                    parent.children.as_mut().unwrap()[idx] = OctreeNode::new();
                }
                let all_empty = parent
                    .children
                    .as_ref()
                    .unwrap()
                    .iter()
                    .all(|c| c.is_empty());
                if all_empty {
                    parent.children = None;
                    parent.is_leaf = true;
                } else {
                    break;
                }
                current = parent;
            }
        }
        true
    }

    /// Grow the octree so that the given world-space point fits within the root.
    /// The previous root becomes a child of the new root without re-inserting every voxel.
    fn expand_root(&mut self, x: f32, y: f32, z: f32) {
        info!("Root expanding ...");

        let old_root = std::mem::replace(&mut self.root, OctreeNode::new());
        let old_center = self.center;
        let half = self.size * 0.5;

        // Determine the direction to shift the center. The old root occupies the opposite child.
        let mut child_index = 0usize;
        if x >= old_center.x {
            self.center.x += half;
        } else {
            self.center.x -= half;
            child_index |= 1;
        }
        if y >= old_center.y {
            self.center.y += half;
        } else {
            self.center.y -= half;
            child_index |= 2;
        }
        if z >= old_center.z {
            self.center.z += half;
        } else {
            self.center.z -= half;
            child_index |= 4;
        }

        self.size *= 2.0;
        self.max_depth += 1;

        let mut children = Box::new(core::array::from_fn(|_| OctreeNode::new()));
        children[child_index] = old_root;
        self.root.children = Some(children);
        self.root.is_leaf = false;

        // Rebuild caches so chunk bookkeeping stays consistent with the new center.
        self.rebuild_cache();
    }

    /// Helper: Collect all voxels from a given octree node iteratively.
    /// The coordinate system here assumes the node covers [–old_size/2, +old_size/2] in each axis.
    fn collect_voxels_from_node(
        node: &OctreeNode,
        old_size: f32,
        center: Vec3,
    ) -> Vec<(Vec3, Voxel, u32)> {
        let mut voxels = Vec::new();
        let mut stack = vec![(
            node,
            center.x - old_size / 2.0,
            center.y - old_size / 2.0,
            center.z - old_size / 2.0,
            old_size,
            0u32,
        )];
        while let Some((n, x, y, z, size, depth)) = stack.pop() {
            if n.is_leaf {
                if let Some(voxel) = n.voxel {
                    let c = Vec3::new(x + size / 2.0, y + size / 2.0, z + size / 2.0);
                    voxels.push((c, voxel, depth));
                }
            }
            if let Some(children) = &n.children {
                let half = size / 2.0;
                for (i, child) in children.iter().enumerate() {
                    let ox = if (i & 1) != 0 { half } else { 0.0 };
                    let oy = if (i & 2) != 0 { half } else { 0.0 };
                    let oz = if (i & 4) != 0 { half } else { 0.0 };
                    stack.push((child, x + ox, y + oy, z + oz, half, depth + 1));
                }
            }
        }
        voxels
    }

    pub fn traverse(&self) -> Vec<(Vec3, u32)> {
        let mut voxels = Vec::new();
        let mut stack = vec![(&self.root, Vec3::splat(0.5), 1.0f32, 0u32)];
        while let Some((node, local_center, size, depth)) = stack.pop() {
            if node.is_leaf {
                if node.voxel.is_some() {
                    voxels.push((self.denormalize_voxel_center(local_center), depth));
                }
            }
            if let Some(children) = &node.children {
                let offset = size / 4.0;
                let new_size = size / 2.0;
                for (i, child) in children.iter().enumerate() {
                    let dx = if (i & 1) != 0 { offset } else { -offset };
                    let dy = if (i & 2) != 0 { offset } else { -offset };
                    let dz = if (i & 4) != 0 { offset } else { -offset };
                    stack.push((
                        child,
                        local_center + Vec3::new(dx, dy, dz),
                        new_size,
                        depth + 1,
                    ));
                }
            }
        }
        voxels
    }

    /// Retrieve a voxel from the octree if it exists (x,y,z in [-0.5..+0.5] range).
    pub fn get_voxel_at(&self, x: f32, y: f32, z: f32) -> Option<&Voxel> {
        Self::get_voxel_recursive(&self.root, x, y, z)
    }

    fn get_voxel_recursive(
        mut node: &OctreeNode,
        mut x: f32,
        mut y: f32,
        mut z: f32,
    ) -> Option<&Voxel> {
        let epsilon = 1e-6;
        loop {
            if node.is_leaf {
                return node.voxel.as_ref();
            }
            let children = node.children.as_ref()?;
            let index = ((x >= 0.5 - epsilon) as usize)
                + ((y >= 0.5 - epsilon) as usize * 2)
                + ((z >= 0.5 - epsilon) as usize * 4);
            let adjust = |coord: f32| {
                if coord >= 0.5 - epsilon {
                    (coord - 0.5) * 2.0
                } else {
                    coord * 2.0
                }
            };
            x = adjust(x);
            y = adjust(y);
            z = adjust(z);
            node = &children[index];
        }
    }

    /// Checks if there is a neighbor voxel at the specified direction from the given world coordinates at the specified depth.
    /// The offsets are directions (-1, 0, 1) for x, y, z.
    pub fn has_neighbor(
        &self,
        position: Vec3,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        depth: u32,
    ) -> bool {
        let aligned = self.normalize_to_voxel_at_depth(position, depth);
        let voxel_count = 2.0_f32.powi(depth as i32);
        // Normalized voxel size is 1/voxel_count
        let norm_voxel_size = 1.0 / voxel_count;

        let neighbor = Vec3::new(
            aligned.x + (offset_x as f32) * norm_voxel_size,
            aligned.y + (offset_y as f32) * norm_voxel_size,
            aligned.z + (offset_z as f32) * norm_voxel_size,
        );

        // Convert the normalized neighbor coordinate back to world space
        let half_size = self.size * 0.5;
        let neighbor_world = neighbor * self.size - Vec3::splat(half_size) + self.center;

        if !self.contains(neighbor_world.x, neighbor_world.y, neighbor_world.z) {
            return false;
        }

        self.get_voxel_at_world_coords(neighbor_world).is_some()
    }

    /// Performs a raycast against the octree and returns the first intersected voxel.
    pub fn raycast(&self, ray: &Ray) -> Option<(f32, f32, f32, u32, Vec3)> {
        // Start from the root node
        let half_size = self.size / 2.0;
        let root_bounds = AABB {
            min: self.center - Vec3::splat(half_size),
            max: self.center + Vec3::splat(half_size),
        };

        let mut stack = vec![(&self.root, root_bounds, 0u32)];
        let mut best: Option<(Vec3, u32, Vec3, f32)> = None; // position, depth, normal, t

        while let Some((node, bounds, depth)) = stack.pop() {
            if let Some((t_enter, _t_exit, normal)) =
                self.ray_intersects_aabb_with_normal(ray, &bounds)
            {
                if node.is_leaf && node.voxel.is_some() {
                    let hit_pos = ray.origin + ray.direction * t_enter;
                    if let Some((_, _, _, best_t)) = best {
                        if t_enter < best_t {
                            best = Some((hit_pos, depth, normal, t_enter));
                        }
                    } else {
                        best = Some((hit_pos, depth, normal, t_enter));
                    }
                }

                if let Some(children) = &node.children {
                    for (i, child) in children.iter().enumerate() {
                        let child_bounds = self.compute_child_bounds(&bounds, i);
                        stack.push((child, child_bounds, depth + 1));
                    }
                }
            }
        }

        best.map(|(pos, depth, normal, _)| (pos.x, pos.y, pos.z, depth, normal))
    }

    /// Save the octree to a file using bincode serialization.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let data = bincode::serialize(self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(path, data)
    }

    /// Load an octree from a file and rebuild runtime caches.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut tree: Self =
            bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        tree.rebuild_cache();
        Ok(tree)
    }

    /// Rebuild runtime caches like occupied_chunks after loading.
    pub fn rebuild_cache(&mut self) {
        self.dirty.clear();
        self.dirty_chunks.clear();
        self.occupied_chunks.clear();

        let voxels = Self::collect_voxels_from_node(&self.root, self.size, self.center);
        for (pos, _voxel, _depth) in voxels {
            let key = chunk_key_from_world(self, pos);
            self.occupied_chunks.insert(key);
        }
    }
}

use super::structure::{SparseVoxelOctree, Voxel, Ray};
use bevy::prelude::*;
use std::path::PathBuf;
use std::io;

#[derive(Component)]
pub struct DiskBackedOctree {
    path: PathBuf,
    max_depth: u32,
    size: f32,
}

impl DiskBackedOctree {
    pub fn new<P: Into<PathBuf>>(path: P, max_depth: u32, size: f32) -> Self {
        Self { path: path.into(), max_depth, size }
    }

    /// Load the octree from disk, execute the closure, then save it back.
    pub fn with_octree<R>(&self, mut f: impl FnMut(&mut SparseVoxelOctree) -> R) -> io::Result<R> {
        let mut tree = if self.path.exists() {
            SparseVoxelOctree::load_from_file(&self.path)?
        } else {
            SparseVoxelOctree::new(self.max_depth, self.size, false, false, false)
        };
        let result = f(&mut tree);
        tree.save_to_file(&self.path)?;
        Ok(result)
    }

    pub fn insert(&self, position: Vec3, voxel: Voxel) -> io::Result<()> {
        self.with_octree(|tree| {
            tree.insert(position, voxel);
        })
    }

    pub fn remove(&self, position: Vec3) -> io::Result<()> {
        self.with_octree(|tree| {
            tree.remove(position);
        })
    }

    pub fn insert_sphere(&self, center: Vec3, radius: i32, voxel: Voxel) -> io::Result<()> {
        self.with_octree(|tree| {
            tree.insert_sphere(center, radius, voxel);
        })
    }

    pub fn remove_sphere(&self, center: Vec3, radius: i32) -> io::Result<()> {
        self.with_octree(|tree| {
            tree.remove_sphere(center, radius);
        })
    }

    pub fn raycast(&self, ray: &Ray) -> io::Result<Option<(f32, f32, f32, u32, Vec3)>> {
        self.with_octree(|tree| tree.raycast(ray))
    }

    pub fn get_voxel_at_world_coords(&self, pos: Vec3) -> io::Result<Option<Voxel>> {
        self.with_octree(|tree| tree.get_voxel_at_world_coords(pos).copied())
    }
}

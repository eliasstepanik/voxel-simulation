use serde::{Serialize, Deserialize};
use bevy::prelude::Vec3;

#[derive(Clone, Serialize, Deserialize)]
pub struct Voxel {
    pub position: [f32; 3],
}

impl Voxel {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.position[0], self.position[1], self.position[2])
    }
}

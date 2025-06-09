use bevy::prelude::*;
use std::collections::VecDeque;
use std::path::Path;
use crate::helper::octree::Octree;
use super::types::Voxel;

fn sort_voxels(queue: &mut VecDeque<Voxel>, camera_pos: Vec3) {
    let mut vec: Vec<Voxel> = queue.drain(..).collect();
    vec.sort_by(|a, b| {
        let da = camera_pos.distance_squared(a.to_vec3());
        let db = camera_pos.distance_squared(b.to_vec3());
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });
    *queue = vec.into();
}

#[derive(Resource)]
pub struct WorldOctree(pub Octree<Voxel>);

#[derive(Resource)]
pub struct LoadingQueue(pub VecDeque<Voxel>);

pub fn load_octree(mut commands: Commands, camera_query: Query<&Transform, With<Camera3d>>) {
    let path = Path::new("assets/world.bin");
    let tree = Octree::<Voxel>::load_from_file(path).unwrap_or_else(|_| Octree::new());
    let mut queue = VecDeque::new();
    tree.collect_data(&mut queue);
    if let Ok(camera) = camera_query.get_single() {
        sort_voxels(&mut queue, camera.translation);
    }
    commands.insert_resource(WorldOctree(tree));
    commands.insert_resource(LoadingQueue(queue));
}

pub fn progressive_load(
    mut commands: Commands,
    mut queue: ResMut<LoadingQueue>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(camera) = camera_query.get_single() {
        sort_voxels(&mut queue.0, camera.translation);
    }
    let per_frame = 5;
    for _ in 0..per_frame {
        if let Some(voxel) = queue.0.pop_front() {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::default()),
                material: materials.add(StandardMaterial::default()),
                transform: Transform::from_translation(voxel.to_vec3()),
                ..default()
            });
        } else {
            break;
        }
    }
}

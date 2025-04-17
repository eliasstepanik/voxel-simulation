use std::collections::HashSet;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{default, Bundle, Commands, Component, Cuboid, DespawnRecursiveExt, Entity, GlobalTransform, Mesh, Query, Res, ResMut, Sphere, Transform};
use bevy_asset::Assets;
use bevy_reflect::Reflect;
use bevy_render::mesh::Mesh3d;
use spacetimedb_sdk::Table;
use crate::module_bindings::{DbTransform, DbVector3, EntityTableAccess, EntityType};
use crate::plugins::network::systems::database::DbConnectionResource;

#[derive(Component)]
pub struct EntityDto {

    pub entity_id: u32,

    
    pub transform: DbTransform,
}



pub fn init(mut commands: Commands,
            ctx: Res<DbConnectionResource>,
            mut meshes: ResMut<Assets<Mesh>>,
            mut materials: ResMut<Assets<StandardMaterial>>,) {



    let debug_material = materials.add(StandardMaterial { ..default() });

    for entity in ctx.0.db.entity().iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default()),),
            MeshMaterial3d(debug_material.clone ()),
            Transform::from_xyz(
                entity.transform.position.x,
                entity.transform.position.y,
                entity.transform.position.z,
            ),
            EntityDto{
                entity_id: entity.entity_id,
                transform: entity.transform
            }
        ));

    }


}



// System that syncs DB entities with the Bevy ECS
pub fn sync_entities_system(
    mut commands: Commands,
    db_resource: Res<DbConnectionResource>,

    // We need the Entity handle for potential despawning,
    // plus mutable references if we want to update Transform/EntityDto
    mut query: Query<(Entity, &mut Transform, &mut EntityDto)>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // --- 1) Collect DB entities and build a set of IDs ---
    let db_entities = db_resource.0.db.entity();
    let db_ids: HashSet<u32> = db_entities.iter().map(|e| e.entity_id).collect();

    // --- 2) For each DB entity, update or spawn in ECS ---
    for db_entity in  db_entities.iter() {

        // Try to find a matching ECS entity by entity_id
        if let Some((_, mut transform, mut dto)) =
            query.iter_mut().find(|(_, _, dto)| dto.entity_id == db_entity.entity_id)
        {
            // Update fields
            dto.transform.position = db_entity.transform.position.clone();
            transform.translation = Vec3::new(
                db_entity.transform.position.x,
                db_entity.transform.position.y,
                db_entity.transform.position.z,
            );

        } else {
            // Not found in ECS, so spawn a new entity
            let debug_material = materials.add(StandardMaterial {
                // fill out any fields you want
                ..default()
            });

            // Pick a mesh based on the entity type
            let entity_type = match db_entity.entity_type {
                EntityType::Sphere => Mesh3d(meshes.add(Sphere::default())),
                EntityType::Cube => Mesh3d(meshes.add(Cuboid::default())),
                EntityType::Custom => todo!(),
            };

            commands.spawn((
                
                Transform::from_xyz(
                    db_entity.transform.position.x,
                    db_entity.transform.position.y,
                    db_entity.transform.position.z,
                ),
                GlobalTransform::default(),
                entity_type,
                MeshMaterial3d(debug_material),
                EntityDto {
                    entity_id: db_entity.entity_id,
                    transform: db_entity.transform.clone(),
                },
            ));
        }

    }

    // --- 3) Despawn any ECS entity that doesn't exist in the DB anymore ---
    for (entity, _, dto) in query.iter_mut() {
        if !db_ids.contains(&dto.entity_id) {
            // This ECS entity no longer matches anything in the DB => remove it
            commands.entity(entity).despawn_recursive();
        }
    }
}
use std::collections::HashSet;
use bevy::log::debug;
use bevy::math::{NormedVectorSpace, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{default, info, Bundle, Commands, Component, Cuboid, DespawnRecursiveExt, DetectChangesMut, Entity, GlobalTransform, Mesh, PbrBundle, Quat, Query, Res, ResMut, Sphere, Transform, TransformBundle};
use bevy_asset::Assets;
use bevy_reflect::Reflect;
use bevy_render::mesh::Mesh3d;
use spacetimedb_sdk::{DbContext, Table};
use crate::helper::math::RoundTo;
use crate::module_bindings::{DbTransform, DbVector3, EntityTableAccess, EntityType, PlayerTableAccess};
use crate::plugins::network::systems::database::DbConnectionResource;

#[derive(Component)]
pub struct EntityDto {

    pub entity_id: u32,

    
    pub transform: DbTransform,
}

impl From<crate::module_bindings::Entity> for EntityDto {
    fn from(e: crate::module_bindings::Entity) -> Self {
        EntityDto {
            entity_id: e.entity_id,
            transform: e.transform,
        }
    }
}




// System that syncs DB entities with the Bevy ECS
pub fn sync_entities_system(
    mut commands: Commands,
    db_resource: Res<DbConnectionResource>,

    // We need the Entity handle for potential despawning,
    // plus mutable references if we want to update Transform/EntityDto
    mut query: Query<(Entity, &mut Transform, &mut GlobalTransform, &mut EntityDto)>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let identity = db_resource.0.identity();
    let player = db_resource.0.db.player().identity().find(&identity);
    
    // --- 1) Collect DB entities and build a set of IDs ---
    let db_entities = db_resource.0.db.entity();
    let db_ids: HashSet<u32> = db_entities.iter().map(|e| e.entity_id).collect();

    // --- 2) For each DB entity, update or spawn in ECS ---
    for db_entity in  db_entities.iter() {

        if db_entity.entity_id == player.clone().unwrap().entity_id {
            return;
        }
        
        // Try to find a matching ECS entity by entity_id
        if let Some((_, mut transform, mut global, mut dto)) =
            query.iter_mut().find(|(_, _, _, dto)| dto.entity_id == db_entity.entity_id)
        {
            // Update fields

            // build the new local Transform
            let new_tf = Transform::from(db_entity.transform.clone());

            // overwrite both components
            *transform = new_tf;
            *global = GlobalTransform::from(new_tf);

            // keep your DTO in sync
            dto.transform = db_entity.transform.clone();

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




            let new_tf = Transform::from(db_entity.transform.clone());

            commands.spawn((
                TransformBundle::from_transform(new_tf),  // inserts BOTH Transform and GlobalTransform
                entity_type,
                MeshMaterial3d(debug_material),
                EntityDto::from(db_entity.clone()),
            ));

        }


    }

    // --- 3) Despawn any ECS entity that doesn't exist in the DB anymore ---
    for (entity,_, _, dto) in query.iter_mut() {
        if !db_ids.contains(&dto.entity_id) {
            // This ECS entity no longer matches anything in the DB => remove it
            commands.entity(entity).despawn_recursive();
        }
    }
}

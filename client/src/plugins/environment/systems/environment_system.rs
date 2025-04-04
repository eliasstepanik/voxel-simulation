
use bevy::prelude::*;
use spacetimedb_sdk::Table;
use crate::module_bindings::{entity_table, DbVector3, EntityTableAccess};
use crate::plugins::network::systems::database::DbConnectionResource;

#[derive(Component)]
pub struct EntityDto {

    pub entity_id: u32,
    pub position: DbVector3,
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
                entity.position.x,
                entity.position.y,
                entity.position.z,
            ),
            EntityDto{
                entity_id: entity.entity_id,
                position: entity.position,
            }
        ));

    }


}



pub fn sync_entities_system(
    mut commands: Commands,
    db_resource: Res<DbConnectionResource>,
    mut query: Query<(Entity, &mut Transform, &mut EntityDto)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ctx = &db_resource.0.db;

    // For each entity record in DB, see if it exists in ECS:
    for db_entity in ctx.entity().iter() {
        // Try to find a matching entity in ECS by comparing `entity_id`
        if let Some((entity, mut transform, mut dto)) = query
            .iter_mut()
            .find(|(_, _, dto)| dto.entity_id == db_entity.entity_id)
        {
            // It already exists. Perhaps update ECS data to match DB:
            dto.position = db_entity.position;
            transform.translation = Vec3::new(
                dto.position.x,
                dto.position.y,
                dto.position.z,
            );
            // ...do any other sync logic
        } else {


            let debug_material = materials.add(StandardMaterial { ..default() });
            // Not found in ECS, so spawn a new entity
            commands.spawn((
                EntityDto {
                    entity_id: db_entity.entity_id,
                    position: db_entity.position.clone(),
                },
                // Create an initial transform using DB data
                Transform::from_xyz(
                    db_entity.position.x,
                    db_entity.position.y,
                    db_entity.position.z,
                ),
                GlobalTransform::default(),
                Mesh3d(meshes.add(Cuboid::default()),),
                MeshMaterial3d(debug_material.clone ()),
            ));
        }
    }
}

pub fn update(time: Res<Time>,)  {

}
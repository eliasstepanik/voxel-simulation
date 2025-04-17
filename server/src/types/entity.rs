
use spacetimedb::{Identity, ReducerContext, SpacetimeType, Table};
use crate::types::types::{DbVector3, DbTransform, DBVector4};

#[spacetimedb::table(name = entity, public)]
#[derive(Debug, Clone, )]
pub struct Entity {
    #[auto_inc]
    #[primary_key]
    pub entity_id: u32,
    pub transform: DbTransform,
    pub entity_type: EntityType,
}

#[derive(SpacetimeType, Clone, Debug)]
pub enum EntityType {
    Cube,
    Sphere,
    Custom
}





#[spacetimedb::reducer]
pub fn spawn_entity(ctx: &ReducerContext, position: DbVector3) -> Result<(), String> {

    ctx.db.entity().try_insert(Entity {
        entity_id: 0,
        transform: DbTransform{
            position: position,
            rotation: DBVector4{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
            scale: DbVector3 {x: 1.0, y: 1.0, z: 1.0 },
            
        },
        entity_type: EntityType::Cube,
    }).expect("TODO: panic message");


    Ok(())
}

use spacetimedb::{Identity, ReducerContext, Table};
use crate::types::vec3::{DbVector3};

#[spacetimedb::table(name = entity, public)]
#[derive(Debug, Clone, )]
pub struct Entity {
    #[auto_inc]
    #[primary_key]
    pub entity_id: u32,


    pub position: DbVector3,

}

#[spacetimedb::reducer]
pub fn spawn_entity(ctx: &ReducerContext, position: DbVector3) -> Result<(), String> {

    ctx.db.entity().try_insert(Entity {
        entity_id: 0,
        position,
    }).expect("TODO: panic message");


    Ok(())
}
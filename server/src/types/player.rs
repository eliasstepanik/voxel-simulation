use spacetimedb::{reducer, Identity, ReducerContext, Table};
use crate::types::entity::{entity, Entity};
use crate::types::vec3::DbVector3;

#[spacetimedb::table(name = player, public)]
#[derive(Debug, Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,

    #[index(btree)]
    pub entity_id: u32,
    
    pub name: Option<String>,
    pub online: bool,
}

#[reducer]
/// Clients invoke this reducer to set their user names.
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = ctx.db.player().identity().find(ctx.sender) {
        ctx.db.player().identity().update(Player { name: Some(name), ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}


#[reducer]
/// Clients invoke this reducer to set their user names.
pub fn set_position(ctx: &ReducerContext, position: DbVector3) -> Result<(), String> {
    if let Some(entity) = ctx.db.entity().iter().find(|e| e.entity_id == ctx.db.player().identity().find(ctx.sender).unwrap().entity_id) {
        ctx.db.entity().entity_id()
            .update(Entity{
            position,
            ..entity
        });
    }
    Ok(())
}

/// Takes a name and checks if it's acceptable as a user's name.
fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}


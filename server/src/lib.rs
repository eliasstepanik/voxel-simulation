mod types;

use std::time::Duration;
use spacetimedb::{reducer, ReducerContext, ScheduleAt, Table, TimeDuration, Timestamp};
use crate::types::entity::{entity, entity__TableHandle, Entity, EntityType};
use crate::types::player::{player, player__TableHandle, Player};
use crate::types::rigidbody::physics_step;
use crate::types::types::{DBVector4, DbTransform, DbVector3};

#[spacetimedb::table(name = config, public)]
pub struct Config {
    #[primary_key]
    pub id: u32,
}

#[spacetimedb::reducer]
pub fn test(ctx: &ReducerContext) -> Result<(), String> {
    log::debug!("This reducer was called by {}.", ctx.sender);
    Ok(())
}


#[reducer(client_connected)]
// Called when a client connects to the SpacetimeDB
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        // If this is a returning player, i.e. we already have a `player` with this `Identity`,
        // set `online: true`, but leave `name` and `identity` unchanged.
        ctx.db.player().identity().update(Player { online: true, ..player });
    } else {
        // If this is a new player, create a `player` row for the `Identity`,
        // which is online, but hasn't set a name.
        let entity = ctx.db.entity().try_insert(Entity{
            entity_id: 0,
            transform: DbTransform{
                position: DbVector3{x: 0.0, y: 0.0, z: 10.0},
                rotation: DBVector4{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
                scale: DbVector3{x: 1.0, y: 1.0, z: 1.0},
            },
                
            entity_type: EntityType::Sphere,
        }).expect("TODO: panic message");

        ctx.db.player().insert(Player {
            name: None,
            identity: ctx.sender,
            online: true,
            entity_id: entity.entity_id,
        });
    }
}

#[reducer(client_disconnected)]
// Called when a client disconnects from SpacetimeDB
pub fn identity_disconnected(ctx: &ReducerContext) {

    let entity: &entity__TableHandle = ctx.db.entity();
    
     
    
    
    if let Some(player_iter) = ctx.db.player().identity().find(ctx.sender) {
        
        
        
        ctx.db.player().identity().update(Player { online: false, ..player_iter });
        ctx.db.entity().iter().find(|e| e.entity_id == player_iter.entity_id).iter().for_each(|e| {
            entity.delete(e.clone());
        })
    } else {
        // This branch should be unreachable,
        // as it doesn't make sense for a client to disconnect without connecting first.
        log::warn!("Disconnect event for unknown Player with identity {:?}", ctx.sender);
    }
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing...");
    ctx.db.config().try_insert(Config {
        id: 0,
    })?;


    ctx.db.physics_timer().try_insert(PhysicsTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(50).into()),
        last_update_ts: ctx.timestamp,
    })?;

    Ok(())
}



#[spacetimedb::table(name = physics_timer, scheduled(physics_step))]
struct PhysicsTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: ScheduleAt,
    last_update_ts: Timestamp,
}

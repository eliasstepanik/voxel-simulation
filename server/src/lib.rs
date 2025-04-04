mod types;

use spacetimedb::{reducer, ReducerContext, SpacetimeType, Table};
use crate::types::player::{player, Player};

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
        ctx.db.player().insert(Player {
            name: None,
            identity: ctx.sender,
            online: true,
        });
    }
}

#[reducer(client_disconnected)]
// Called when a client disconnects from SpacetimeDB
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        ctx.db.player().identity().update(Player { online: false, ..player });
    } else {
        // This branch should be unreachable,
        // as it doesn't make sense for a client to disconnect without connecting first.
        log::warn!("Disconnect event for unknown Player with identity {:?}", ctx.sender);
    }
}
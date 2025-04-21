use std::fmt::Debug;
use std::ops::Deref;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Commands, DetectChanges, Mut, Res, ResMut, Resource, World};
use bevy::utils::info;
use spacetimedb_sdk::{credentials, DbContext, Error, Event, Identity, Status, Table, TableWithPrimaryKey};
use crate::config::ServerConfig;
use crate::module_bindings::*;
use crate::plugins::network::systems::callbacks::*;
use crate::plugins::network::systems::connection::*;
use crate::plugins::network::systems::subscriptions::*;

/// The URI of the SpacetimeDB instance hosting our chat module.
const HOST: &str = "http://100.85.241.101:3000";

/// The database name we chose when we published our module.
const DB_NAME: &str = "network-game";

#[derive(Resource)]
pub struct DbConnectionResource(pub(crate) DbConnection);

pub fn setup_database(mut commands: Commands, config: Res<crate::Config>) {
    // Call your connection function and insert the connection as a resource.
    let ctx = connect_to_db(config);
    register_callbacks(&ctx);
    subscribe_to_tables(&ctx);
    ctx.run_threaded();
    commands.insert_resource(DbConnectionResource(ctx));
    

}


/// Register subscriptions for all rows of both tables

fn connect_to_db(config: Res<crate::Config>) -> DbConnection {

    println!("It's there: {:?}", &config.server);

    DbConnection::builder()
        .on_connect(on_connected)
        .on_connect_error(on_connect_error)
        .on_disconnect( on_disconnected)
        .with_module_name(&config.server.database)
        .with_uri(&config.server.host)
        .build()
        .expect("Failed to connect")
}



/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks(ctx: &DbConnection) {
    // When a new user joins, print a notification.
    ctx.db.player().on_insert(on_user_inserted);

    // When a user's status changes, print a notification.
    ctx.db.player().on_update(on_user_updated);

    // When we fail to set our name, print a warning.
    ctx.reducers.on_set_name(on_name_set);
}

fn subscribe_to_tables(ctx: &DbConnection) {
    ctx.subscription_builder()
        .on_applied(on_sub_applied)
        .on_error(on_sub_error)
        .subscribe(["SELECT * FROM player", "SELECT * FROM entity"]);
}
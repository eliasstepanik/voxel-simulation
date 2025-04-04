use bevy::prelude::info;
use spacetimedb_sdk::{Error, Table};
use crate::module_bindings::{ErrorContext, PlayerTableAccess, SubscriptionEventContext};

/// Our `on_subscription_applied` callback:
/// sort all past messages and print them in timestamp order.
pub fn on_sub_applied(ctx: &SubscriptionEventContext) {

    let mut players = ctx.db.player().iter().collect::<Vec<_>>();
    players.sort_by_key(|p| p.name.clone());
    for player in players {
        println!("Player {:?} online", player.name);
    }
    println!("Fully connected and all subscriptions applied.");
    println!("Use /name to set your name, or type a message!");
}

/// Or `on_error` callback:
/// print the error, then exit the process.
pub fn on_sub_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Subscription failed: {}", err);
    std::process::exit(1);
}
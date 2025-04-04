use bevy::log::{error, info};
use spacetimedb_sdk::Status;
use crate::module_bindings::{EventContext, Player, ReducerEventContext, RemoteDbContext};

/// Our `User::on_insert` callback:
/// if the user is online, print a notification.
pub fn on_user_inserted(_ctx: &EventContext, user: &Player) {
    if user.online {
        info!("User {} connected.", user_name_or_identity(user));
    }
}

pub fn user_name_or_identity(user: &Player) -> String {
    user.name
        .clone()
        .unwrap_or_else(|| user.identity.to_hex().to_string())
}

/// Our `User::on_update` callback:
/// print a notification about name and status changes.
pub fn on_user_updated(_ctx: &EventContext, old: &Player, new: &Player) {
    if old.name != new.name {
        info!(
            "User {} renamed to {}.",
            user_name_or_identity(old),
            user_name_or_identity(new)
        );
    }
    if old.online && !new.online {
        info!("User {} disconnected.", user_name_or_identity(new));
    }
    if !old.online && new.online {
        info!("User {} connected.", user_name_or_identity(new));
    }
}


/// Our `on_set_name` callback: print a warning if the reducer failed.
pub fn on_name_set(ctx: &ReducerEventContext, name: &String) {
    if let Status::Failed(err) = &ctx.event.status {
        error!("Failed to change name to {:?}: {}", name, err);
    }
}

/// Our `on_send_message` callback: print a warning if the reducer failed.
pub fn on_message_sent(ctx: &ReducerEventContext, text: &String) {
    if let Status::Failed(err) = &ctx.event.status {
        error!("Failed to send message {:?}: {}", text, err);
    }
}

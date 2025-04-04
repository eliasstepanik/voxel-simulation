use bevy::app::{App, Plugin, Startup};
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use crate::plugins::environment::systems::environment_system::*;
use crate::plugins::network::systems::database::setup_database;

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_database);
    }
}

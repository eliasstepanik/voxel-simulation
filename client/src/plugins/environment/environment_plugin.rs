use bevy::app::{App, Plugin, Startup};
use bevy::color::palettes::basic::{GREEN, YELLOW};
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use crate::plugins::environment::systems::environment_system::*;
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
        app.add_systems(FixedUpdate, fixed_update);
        app.add_systems(Update, fixed_update);
    }
}

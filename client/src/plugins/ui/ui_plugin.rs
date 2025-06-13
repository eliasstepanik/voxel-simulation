use crate::plugins::ui::systems::ui_system::*;
use bevy::app::{App, FixedUpdate, Plugin, PreUpdate, Startup};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, update);
    }
}

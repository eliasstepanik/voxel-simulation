
use bevy::app::{App, Plugin, PreUpdate, Startup};
use bevy::prelude::{IntoSystemConfigs, Update};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, _app: &mut App) {
        _app.add_systems(
            Update,
            (
                crate::plugins::input::systems::console::console_system,
                crate::plugins::input::systems::flight::flight_systems,
                crate::plugins::input::systems::ui::ui_system,
                crate::plugins::input::systems::network::network_system,
                crate::plugins::input::systems::movement::movement_system,
            ),

        );
    }
}

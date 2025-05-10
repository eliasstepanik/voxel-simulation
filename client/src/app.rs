use crate::helper::debug_gizmos::debug_gizmos;
use bevy::prelude::*;
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::plugins::ui::ui_plugin::UiPlugin);
        app.add_plugins(crate::plugins::environment::environment_plugin::EnvironmentPlugin);
        //app.add_plugins(crate::plugins::network::network_plugin::NetworkPlugin);
        app.add_plugins(crate::plugins::input::input_plugin::InputPlugin);

        app.add_systems(Update, (debug_gizmos));
        app.register_type::<Option<Handle<Image>>>();
        app.register_type::<AlphaMode>();
    }
}

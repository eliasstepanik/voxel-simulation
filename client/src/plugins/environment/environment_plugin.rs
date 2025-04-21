use bevy::app::{App, Plugin, PreStartup, PreUpdate, Startup};
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Startup,
            (crate::plugins::environment::systems::camera_system::setup),
        );
    }
}

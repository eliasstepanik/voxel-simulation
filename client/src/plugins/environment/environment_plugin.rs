use bevy::app::{App, Plugin, Startup, Update};
use crate::plugins::world::systems::{load_octree, progressive_load};
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Startup,
            (
                crate::plugins::environment::systems::environment_system::setup,
                crate::plugins::environment::systems::camera_system::setup,
                load_octree,
            ),
        );
        app.add_systems(Update, progressive_load);
    }
}

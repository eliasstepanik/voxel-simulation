use bevy::app::{App, Plugin, PreStartup, PreUpdate, Startup};
use bevy::prelude::IntoSystemConfigs;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Startup,
            (crate::plugins::environment::systems::camera_system::setup,crate::plugins::environment::systems::environment_system::setup.after(crate::plugins::environment::systems::camera_system::setup) ),
        );

        
    }
}

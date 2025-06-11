use bevy::pbr::wireframe::WireframePlugin;
use crate::helper::debug_gizmos::debug_gizmos;
use bevy_easy_compute::prelude::{AppComputePlugin, AppComputeWorkerPlugin};
use crate::plugins::environment::systems::voxels::sphere_compute::SphereWorker;
use crate::plugins::environment::systems::voxels::visible_chunks_compute::VisibleChunksWorker;
use bevy::prelude::*;
pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::plugins::ui::ui_plugin::UiPlugin);
        app.add_plugins(crate::plugins::big_space::big_space_plugin::BigSpaceIntegrationPlugin);
        app.add_plugins(crate::plugins::environment::environment_plugin::EnvironmentPlugin);
        app.add_plugins(AppComputePlugin);
        app.add_plugins(AppComputeWorkerPlugin::<SphereWorker>::default());
        app.add_plugins(AppComputeWorkerPlugin::<VisibleChunksWorker>::default());
        //app.add_plugins(crate::plugins::network::network_plugin::NetworkPlugin);
        app.add_plugins(crate::plugins::input::input_plugin::InputPlugin);
        app.add_plugins(WireframePlugin);

        app.add_systems(Update, (debug_gizmos));
        app.register_type::<Option<Handle<Image>>>();
        app.register_type::<AlphaMode>();
    }
}

mod app;
mod helper;
mod plugins;
mod config;

use std::fs;
use crate::app::AppPlugin;
use bevy::gizmos::{AppGizmoBuilder, GizmoPlugin};
use bevy::log::info;
use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::DefaultPlugins;
use bevy::input::gamepad::AxisSettingsError::DeadZoneUpperBoundGreaterThanLiveZoneUpperBound;
use bevy::window::PresentMode;
use big_space::plugin::BigSpaceDefaultPlugins;
use toml;
use crate::config::Config;
use crate::plugins::big_space::big_space_plugin::BigSpaceIntegrationPlugin;

const TITLE: &str = "voxel-simulation";
const RESOLUTION: (f32, f32) = (1920f32, 1080f32);
const RESIZABLE: bool = true;
const DECORATIONS: bool = true;
const TRANSPARENT: bool = true;
const PRESENT_MODE: PresentMode = PresentMode::AutoVsync;



fn main() {
    let config_str = fs::read_to_string("Config.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config");




    let mut app = App::new();

    app.insert_resource(config);

    register_platform_plugins(&mut app);

    app.add_plugins(AppPlugin);




    /*app.add_plugins(GizmoPlugin);*/

    app.run();
}

#[derive(Resource)]
pub struct InspectorVisible(bool);
fn register_platform_plugins(app: &mut App) {
    #[cfg(target_os = "windows")]
    {
        // Register Windows-specific plugins
        info!("Adding Windows-specific plugins");
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),      // Window title
                        resolution: RESOLUTION.into(), // Initial resolution (width x height)
                        resizable: RESIZABLE,          // Allow resizing
                        decorations: DECORATIONS,      // Enable window decorations
                        transparent: TRANSPARENT,      // Opaque background
                        present_mode: PRESENT_MODE,    // VSync mode
                        ..default()
                    }),
                    ..default()
                }).build().disable::<TransformPlugin>(),
        );
    }

    #[cfg(target_os = "macos")]
    {
        info!("Adding macOS-specific plugins");
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: crate::TITLE.to_string(),      // Window title
                resolution: crate::RESOLUTION.into(), // Initial resolution (width x height)
                resizable: crate::RESIZABLE,          // Allow resizing
                decorations: crate::DECORATIONS,      // Enable window decorations
                transparent: crate::TRANSPARENT,      // Opaque background
                present_mode: crate::PRESENT_MODE,    // VSync mode
                ..default()
            }),
            ..default()
        }));
    }
}
fn should_display_inspector(inspector_visible: Res<InspectorVisible>) -> bool {
    inspector_visible.0
}

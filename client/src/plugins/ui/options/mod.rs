use crate::plugins::environment::systems::voxels::structure::SparseVoxelOctree;
use crate::plugins::input::systems::voxels::VoxelEditMode;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Resource)]
pub struct OptionsMenuConfig {
    pub items: Vec<OptionItem>,
}

#[derive(Deserialize, Clone)]
pub struct OptionItem {
    pub label: String,
    pub action: String,
}

#[derive(Component)]
struct MenuButton {
    action: String,
}

pub struct OptionsMenuPlugin;
impl Plugin for OptionsMenuPlugin {
    fn build(&self, app: &mut App) {
        let config: OptionsMenuConfig = load_config();
        app.insert_resource(config);
        app.add_systems(Startup, setup_menu);
        app.add_systems(Update, handle_buttons);
    }
}

fn load_config() -> OptionsMenuConfig {
    let path = "client/assets/options_menu.json";
    let data = std::fs::read_to_string(path).expect("failed to read options_menu.json");
    serde_json::from_str(&data).expect("invalid options_menu.json")
}

fn setup_menu(
    mut commands: Commands,
    config: Res<OptionsMenuConfig>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            for item in config.items.iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(2.0)),
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            background_color: Color::GRAY.into(),
                            ..default()
                        },
                        MenuButton {
                            action: item.action.clone(),
                        },
                    ))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            item.label.clone(),
                            TextStyle {
                                font: asset_server.load("fonts/minecraft_font.ttf"),
                                font_size: 16.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            }
        });
}

fn handle_buttons(
    mut interactions: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut octrees: Query<&mut SparseVoxelOctree>,
    mut exit: EventWriter<AppExit>,
    mut edit_mode: ResMut<VoxelEditMode>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            match button.action.as_str() {
                "toggle_wireframe" => {
                    for mut o in &mut octrees {
                        o.show_wireframe = !o.show_wireframe;
                    }
                }
                "toggle_grid" => {
                    for mut o in &mut octrees {
                        o.show_world_grid = !o.show_world_grid;
                    }
                }
                "toggle_edit_mode" => {
                    *edit_mode = match *edit_mode {
                        VoxelEditMode::Single => VoxelEditMode::Sphere,
                        VoxelEditMode::Sphere => VoxelEditMode::Single,
                    };
                }
                "exit" => {
                    exit.send(AppExit);
                }
                _ => {}
            }
        }
    }
}

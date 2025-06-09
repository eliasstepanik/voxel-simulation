use crate::plugins::environment::systems::camera_system::CameraController;
use bevy::asset::AssetServer;
use bevy::math::DVec3;
use bevy::prelude::*;

use big_space::prelude::*;

#[derive(Component)]
pub struct SpeedDisplay;

/// Spawns a UI Text entity to display speed/positions.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Use the new UI API, or the old UI Node-based system.
    // This example uses an older approach to Node/Style, but can be adapted to `TextBundle`.
    // If you're on Bevy 0.11+, you can also do `TextBundle::from_section(...)`.
    commands.spawn((
        // The text to display:
        Text::new("Speed: 0.0"),
        // The font, loaded from an asset file
        TextFont {
            font: asset_server.load("fonts/minecraft_font.ttf"),
            font_size: 25.0,
            ..default()
        },
        // The text layout style
        TextLayout::new_with_justify(JustifyText::Left),
        // Style for positioning the UI node
        Node {
            position_type: PositionType::Relative,
            bottom: Val::Px(9.0),
            right: Val::Px(9.0),
            ..default()
        },
        // Our marker so we can query this entity
        SpeedDisplay,
    ));
}

/// System that updates the UI text each frame with
///  - speed
///  - camera f32 position
///  - camera global f64 position
///  - current chunk coordinate

pub fn update(
    grids: Grids<'_, '_, i64>,         // helper from big_space
    // we need the entity id, the cell & the local transform
    camera_q: Query<(Entity, &GridCell<i64>, &Transform, &CameraController)>,
    mut ui_q:  Query<&mut Text, With<SpeedDisplay>>,
) {
    let Ok((cam_ent, cell, tf, ctrl)) = camera_q.get_single() else { return };

    // grid that the camera lives in
    let Some(grid) = grids.parent_grid(cam_ent) else { return };

    // absolute position in metres (f64)
    let pos = grid.grid_position_double(cell,tf);

    if let Ok(mut text) = ui_q.get_single_mut() {
        text.0 = format!(
            "\n  Speed: {:.3}\n  Position(f64): ({:.2}, {:.2}, {:.2})",
            ctrl.speed,
            pos.x, pos.y, pos.z,
        );
    }
}
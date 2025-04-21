use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{EventReader, EventWriter, KeyCode, Query, Res, ResMut, Time, Transform};
use bevy_window::Window;
use crate::plugins::environment::systems::camera_system::CameraController;
use crate::plugins::network::systems::database::DbConnectionResource;


///TODO
pub fn movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>, /*
                                               mouse_button_input: Res<ButtonInput<MouseButton>>,*/
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut windows: Query<&mut Window>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut ctx: ResMut<DbConnectionResource>,
) {
}
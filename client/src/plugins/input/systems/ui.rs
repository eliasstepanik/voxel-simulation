use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

pub fn ui_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        // Toggle between locked and unlocked
        if window.cursor_options.grab_mode == CursorGrabMode::None {
            // Lock
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        } else {
            // Unlock
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
    
    if keyboard_input.pressed(KeyCode::Escape) {
        app_exit_events.write(Default::default());
    }
}
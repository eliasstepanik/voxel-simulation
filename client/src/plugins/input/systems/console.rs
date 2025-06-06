use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::plugins::environment::systems::camera_system::CameraController;
pub fn console_system(
    mut ctxs: EguiContexts,
    mut state: ResMut<ConsoleState>,
) {
    if !state.open { return; }

    egui::Window::new("Console")
        .resizable(true)
        .vscroll(true)
        .show(ctxs.ctx_mut(), |ui| {
            // Output
            for line in &state.output {
                ui.label(line);
            }

            // Input line
            let resp = ui.text_edit_singleline(&mut state.input);
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let cmd = state.input.trim().to_string();
                if !cmd.is_empty() {
                    state.history.push(cmd.clone());
                    handle_command(&cmd, &mut state.output);
                    state.input.clear();
                }
            }
        });
}
/// Press ` to open / close
pub fn toggle_console(
    mut state: ResMut<ConsoleState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        state.open = !state.open;
    }
}

/// Add your own commands here.
/// For demo purposes we just echo the input.
fn handle_command(cmd: &str, out: &mut Vec<String>) {
    match cmd.trim() {
        "help" => out.push("Available: help, clear, echo …".into()),
        "clear" => out.clear(),
        _ => out.push(format!("> {cmd}")),
    }
}

#[derive(Resource, Default)]
pub struct ConsoleState {
    pub open: bool,
    pub input: String,
    pub history: Vec<String>,
    pub output: Vec<String>,
}

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use crate::io::IoDevices;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, ui_example_system);
        app.add_systems(Startup, setup_camera_system);
    }
}

fn setup_camera_system(mut commands: Commands) {
    // commands.spawn(Camera2d);
}
fn ui_example_system(mut contexts: EguiContexts, mut io: ResMut<IoDevices>) -> Result {
    egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
        ui.label("Digital Inputs");
        for (key, signals) in io.inputs.iter_mut() {
            ui.label(format!("Device: {key}"));
            for (idx, mut bit) in signals.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // ui.label(format!("I: {idx}"));
                    ui.checkbox(&mut bit, format!("IX {}.{}", idx / 8, idx % 8));
                });
            }
        }
    });
    Ok(())
}

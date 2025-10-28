use std::io::Read;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass},
    egui::WidgetText,
};
use bitvec::prelude::BitVec;

use crate::io::{DioPin, IOStore, Io, IoDevices, NodeId, UIOveride};
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, dio_ui);
    }
}

fn dio_ui(mut cmd: Commands, mut contexts: EguiContexts, mut io: ResMut<IoDevices>) -> Result {
    egui::Window::new("IO Devices")
        .scroll([true, true])
        .show(contexts.ctx_mut()?, |ui| {
            // egui::ScrollArea::vertical().show(ui, |ui| {});

            ui.label("Digital Inputs");
            io_widget(&mut cmd, ui, &mut io.digital_inputs, "input", Io::Input);
            ui.label("Digital Outputs");
            io_widget(&mut cmd, ui, &mut io.digital_outputs, "output", Io::Output);
        });
    Ok(())
}

fn io_widget(
    cmd: &mut Commands,
    ui: &mut egui::Ui,
    hash_map: &mut HashMap<NodeId, IOStore>,
    header: &'static str,
    kind: Io,
) {
    for (address, store) in hash_map.iter_mut() {
        ui.collapsing(format!("{} Device: {}", header, address.0), |ui| {
            let bytes = store.state.chunks_exact_mut(8);
            for (address, byte) in bytes.enumerate() {
                ui.horizontal_top(|ui| {
                    ui.collapsing(format!("B {address}"), |ui| {
                        for (ix, mut bit) in byte.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                // ui.label(format!("bit: {ix}"));
                                if ui.checkbox(&mut bit, format!(".{ix}")).changed() {
                                    cmd.trigger(UIOveride {
                                        address: (address as u32).into(),
                                        pin: DioPin(ix as u16),
                                        value: *bit,
                                        kind,
                                    });
                                }
                            });
                        }
                    });
                    let byte: u8 = byte.bytes().next().unwrap().unwrap();
                    ui.label(format!("{:#04X}", byte))
                });
            }
        });
    }
}

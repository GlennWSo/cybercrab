use std::io::Read;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bitvec::field::BitField;

use crate::io::{DIOPin, DigitalInputSet, IoDevices};
use itertools::Itertools;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, dio_ui);
    }
}

fn dio_ui(mut cmd: Commands, mut contexts: EguiContexts, mut io: ResMut<IoDevices>) -> Result {
    egui::Window::new("Hello")
        .scroll([true, true])
        .show(contexts.ctx_mut()?, |ui| {
            // egui::ScrollArea::vertical().show(ui, |ui| {});

            ui.label("Digital Inputs");
            for (address, signals) in io.digital_inputs.iter_mut() {
                ui.collapsing(format!("Device: {}", address.0), |ui| {
                    let bytes = signals.chunks_exact_mut(8);
                    for (address, byte) in bytes.enumerate() {
                        ui.horizontal_top(|ui| {
                            ui.collapsing(format!("B {address}"), |ui| {
                                for (ix, mut bit) in byte.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        // ui.label(format!("bit: {ix}"));
                                        if ui.checkbox(&mut bit, format!(".{ix}")).changed() {
                                            cmd.trigger(DigitalInputSet {
                                                address: (address as u32).into(),
                                                pin: DIOPin(ix as u16),
                                                value: *bit,
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
        });
    Ok(())
}

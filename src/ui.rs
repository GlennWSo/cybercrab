use std::io::Read;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bitvec::field::BitField;

use crate::io::IoDevices;
use itertools::Itertools;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, ui_example_system);
    }
}

fn ui_example_system(mut contexts: EguiContexts, mut io: ResMut<IoDevices>) -> Result {
    egui::Window::new("Hello")
        .scroll([true, true])
        .show(contexts.ctx_mut()?, |ui| {
            // egui::ScrollArea::vertical().show(ui, |ui| {});

            ui.label("Digital Inputs");
            for (key, signals) in io.inputs.iter_mut() {
                ui.collapsing(format!("Device: {key}"), |ui| {
                    let bytes = signals.chunks_exact_mut(8);
                    for (address, byte) in bytes.enumerate() {
                        ui.horizontal_top(|ui| {
                            ui.collapsing(format!("B {address}"), |ui| {
                                for (ix, mut bit) in byte.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        // ui.label(format!("bit: {ix}"));
                                        ui.checkbox(&mut bit, format!(".{ix}"));
                                    });
                                }
                            });
                            let byte: u8 = byte.bytes().next().unwrap().unwrap();
                            ui.label(format!("{:#04X}", byte))
                        });
                    }

                    // for (address, byte) in &mut signals.iter_mut().chunks(8).into_iter().iter() {
                    //     for (bit_i, bit) in byte.into_iter().enumerate() {
                    //         ui.horizontal(|ui| {
                    //             ui.checkbox(&mut bit, format!("IX {}.{}", idx / 8, idx % 8));
                    //         });
                    //     }
                    // }
                    // for (idx, mut bit) in signals.iter_mut().enumerate() {
                    //     ui.horizontal(|ui| {
                    //         ui.checkbox(&mut bit, format!("IX {}.{}", idx / 8, idx % 8));
                    //     });
                    // }
                });
            }
        });
    Ok(())
}

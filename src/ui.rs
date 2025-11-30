use std::io::Read;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass},
    egui::WidgetText,
};
use bitvec::prelude::BitVec;
use itertools::Itertools;

use crate::{
    io::Address,
    shiftreg::{Register, RegisterPosition},
    tbana::{SwitchDirection, TransportState},
};
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, monitor_state);
    }
}

fn monitor_state(mut cmd: Commands, mut contexts: EguiContexts) -> Result {
    egui::Window::new("PLC state")
        .scroll([true, true])
        .show(contexts.ctx_mut()?, |ui| "derp");
    Ok(())
}

use bevy::prelude::*;

pub mod fotocell;
pub mod io;
pub mod motor;
pub mod physics;
pub mod plc;
pub mod sensor;
pub mod shiftreg;
pub mod spawn;
mod sysorder;
mod tbana;
pub mod ui;
use avian3d::prelude::PhysicsPlugins;
pub use sysorder::InitSet;
pub use tbana::TbanaPlugin;

use crate::{
    fotocell::FotocellPlugin, io::IoPlugin, motor::MotorPlugin, shiftreg::ShiftRegPlugin,
    spawn::SpawnPlugin, sysorder::SysOrderPlugin, ui::UIPlugin,
};

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TbanaPlugin);
        app.add_plugins(IoPlugin);
        app.add_plugins(FotocellPlugin);
        app.add_plugins(UIPlugin);
        app.add_plugins(SysOrderPlugin);
        app.add_plugins(ShiftRegPlugin);
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(MotorPlugin);
        app.add_plugins(SpawnPlugin);
    }
}

pub enum SimulationState {
    Running,
    Paused,
}

pub enum ImmersionState {}

pub enum MachineState {
    Auto,
    Man { bypass: bool },
    Standby,
    EmergancyStop,
    Ugl,
}

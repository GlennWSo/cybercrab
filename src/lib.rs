use bevy::prelude::*;

pub mod io;
pub mod shiftreg;
mod tbana;
pub use tbana::TbanaPlugin;

use tbana::TbanaBundle;

use crate::{
    io::{
        Address, AttachedThings, ConnectedTo, DeviceNetwork, FotoCell, IoPlugin, IoSlot, IoThing,
    },
    tbana::PushTo,
};

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TbanaPlugin);
        app.add_plugins(IoPlugin);
        app.add_systems(Startup, spawn_some_stuff);
    }
}

fn spawn_some_stuff(mut cmd: Commands, mut net: ResMut<DeviceNetwork>) {
    let device_address: Address = 1;
    let device_id = io::InputDevice::<2>::spawn(&mut cmd, &mut net, "inputmodule1", device_address);

    for i in 1..=4 {
        let fotocell = (
            FotoCell,
            Name::new(format!("fotocell{i}")),
            ConnectedTo(device_id),
            IoSlot::new(0, io::DataSlice::Bit(i)),
        );
        cmd.spawn(fotocell);
    }

    let n = 8;

    let last_bundle = TbanaBundle::new(format!("Stn: {}", n));
    let mut id = cmd.spawn(last_bundle).id();
    for i in (1..n).rev() {
        let bundle = (TbanaBundle::new(format!("Stn: {}", i)), PushTo(id));
        id = cmd.spawn(bundle).id();
    }
}

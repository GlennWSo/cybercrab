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

    let fotocells: Vec<_> = (1..=12)
        .map(|i| {
            let ptr = i / 8;
            let idx = i % 8;
            let fotocell = (
                FotoCell,
                Name::new(format!("fotocell{i}")),
                IoSlot::new(ptr, io::DataSlice::Bit(idx)),
            );
            cmd.spawn(fotocell).id()
        })
        .collect();

    cmd.entity(device_id).add_related::<ConnectedTo>(&fotocells);
    let n = 3;

    let last_bundle = TbanaBundle::new(format!("Stn: {}", n + 1));
    let mut id = cmd.spawn(last_bundle).add_children(&fotocells[0..4]).id();
    for i in (1..n).rev() {
        let bundle = (TbanaBundle::new(format!("Stn: {}", i + 1)), PushTo(id));
        let foto_idx = i * 4;
        let children = &fotocells[foto_idx..(foto_idx + 4)];
        id = cmd.spawn(bundle).add_children(children).id();
    }
}

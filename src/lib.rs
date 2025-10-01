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
    tbana::{load_assets, PushTo, TBanaAssets},
};

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TbanaPlugin);
        app.add_plugins(IoPlugin);
        app.add_systems(Startup, spawn_some_stuff.after(load_assets));
    }
}

fn spawn_some_stuff(
    mut cmd: Commands,
    mut net: ResMut<DeviceNetwork>,
    tbana_assets: Res<TBanaAssets>,
) {
    let device_address: Address = 1;
    const SIZE: usize = 4;
    // spawn input node with SIZE*8 bits
    let device_id =
        io::DigitalInputDevice::<SIZE>::spawn(&mut cmd, &mut net, "inputmodule1", device_address);

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

    let mut translation = Vec3::default();
    let last_bundle = TbanaBundle::new(format!("Stn: {}", n + 1), &tbana_assets);
    let mut id = cmd
        .spawn((last_bundle, Transform::from_translation(translation)))
        .add_children(&fotocells[0..4])
        .id();

    let spaceing = 6.0;
    for i in (1..n).rev() {
        translation.z += spaceing;
        let bundle = (
            TbanaBundle::new(format!("Stn: {}", i + 1), &tbana_assets),
            Transform::from_translation(translation),
            PushTo(id),
        );
        let foto_idx = i * 4;
        let children = &fotocells[foto_idx..(foto_idx + 4)];
        id = cmd.spawn(bundle).add_children(children).id();
    }
}

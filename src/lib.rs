use bevy::prelude::*;

pub mod fotocell;
pub mod io;
pub mod shiftreg;
mod tbana;
pub use tbana::TbanaPlugin;

use tbana::TbanaBundle;

use crate::{
    fotocell::{Fotocell, FotocellAssets, FotocellBundle, FotocellPlugin},
    io::{Address, ConnectedTo, DeviceNetwork, IoPlugin, IoSlot},
    tbana::{load_assets, PushTo, TBanaAssets},
};

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TbanaPlugin);
        app.add_plugins(IoPlugin);
        app.add_plugins(FotocellPlugin);
        app.add_systems(Startup, spawn_some_stuff.after(load_assets));
    }
}

fn spawn_some_stuff(
    mut cmd: Commands,
    mut net: ResMut<DeviceNetwork>,
    fotocell_assets: Res<FotocellAssets>,
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

            let z = (i % 2) as f32 * 0.2 - 0.1 + ((i % 4) / 2) as f32 * 1.6 - 0.8;
            let coord = Vec3 { x: 0.5, y: 0.6, z };
            let mut transform = Transform::from_translation(coord);
            transform.rotate_local_y(90_f32.to_radians());

            let name = format!("fotocell_{i}");
            let io_slot = IoSlot::new(ptr, io::DataSlice::Bit(idx));
            let fotocell = FotocellBundle::new(name, io_slot, &fotocell_assets, device_id);
            cmd.spawn((fotocell, transform)).id()
        })
        .collect();

    // cmd.entity(device_id).add_related::<ConnectedTo>(&fotocells);
    let n = 3;

    let mut translation = Vec3::default();
    let last_bundle = TbanaBundle::new(format!("Stn: {}", n + 1), &tbana_assets);
    let mut id = cmd
        .spawn((last_bundle, Transform::from_translation(translation)))
        .add_children(&fotocells[0..4])
        .id();

    let spaceing = 2.1;
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

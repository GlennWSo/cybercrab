use bevy::prelude::*;

pub mod fotocell;
pub mod io;
pub mod shiftreg;
mod sysorder;
mod tbana;
pub mod ui;
use avian3d::prelude::PhysicsPlugins;
pub use sysorder::InitSet;
pub use tbana::TbanaPlugin;

use tbana::TbanaBundle;

use crate::{
    fotocell::{
        on_fotocell_blocked, on_fotocell_unblocked, on_laser_color, FotocellAssets, FotocellBundle,
        FotocellPlugin, LaserBundle,
    },
    io::{DIOPin, DeviceNetwork, IoDevices, IoPlugin, NetAddress},
    shiftreg::{Detail, ShiftRegPlugin},
    sysorder::SysOrderPlugin,
    tbana::{PushTo, TBanaAssets},
    ui::UIPlugin,
};

use bitvec::prelude::*;
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
        app.add_systems(Startup, spawn_some_stuff.in_set(InitSet::Spawn));
    }
}

fn spawn_some_stuff(
    mut cmd: Commands,
    fotocell_assets: Res<FotocellAssets>,
    tbana_assets: Res<TBanaAssets>,
    mut io: ResMut<IoDevices>,
) {
    let device_address: NetAddress = 0;
    io.inputs.insert(device_address, bitvec![u32, Lsb0; 0; 32]);

    let fotocells: Vec<_> = (1..=12)
        .map(|i| {
            let z = (i % 2) as f32 * 0.2 - 0.1 + ((i % 4) / 2) as f32 * 1.6 - 0.8;
            let coord = Vec3 {
                x: 0.45,
                y: 0.53,
                z,
            };
            let mut transform = Transform::from_translation(coord);
            transform.rotate_local_y(-90_f32.to_radians());

            let name = format!("fotocell_{i}");
            let io_slot = DIOPin(i - 1);
            let fotocell =
                FotocellBundle::new(name, io_slot, &fotocell_assets, device_address, 0.8);
            let laser = LaserBundle::new(&fotocell_assets);
            let laser = cmd.spawn(laser).observe(on_laser_color).id();
            let fotocell = cmd
                .spawn((fotocell, transform))
                .observe(on_fotocell_blocked)
                .observe(on_fotocell_unblocked)
                .id();
            cmd.entity(fotocell).add_child(laser);
            fotocell
        })
        .collect();

    // cmd.entity(device_id).add_related::<ConnectedTo>(&fotocells);
    let n = 3;

    let mut translation = Vec3::default();
    let last_bundle = TbanaBundle::new(format!("Stn: {}", n), &tbana_assets);
    let mut id = cmd
        .spawn((last_bundle, Transform::from_translation(translation)))
        .add_children(&fotocells[0..4])
        .id();

    let spaceing = 2.1;
    for i in (1..n).rev() {
        translation.z += spaceing;
        let bundle = (
            TbanaBundle::new(format!("Stn: {}", i), &tbana_assets),
            Transform::from_translation(translation),
            PushTo(id),
        );
        let foto_idx = i * 4;
        let children = &fotocells[foto_idx..(foto_idx + 4)];
        id = cmd.spawn(bundle).add_children(children).id();
    }
}

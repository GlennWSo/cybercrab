use bevy::prelude::*;

pub mod fotocell;
pub mod io;
pub mod plc;
pub mod sensor;
pub mod shiftreg;
mod sysorder;
mod tbana;
pub mod ui;
use avian3d::prelude::PhysicsPlugins;
use itertools::Itertools;
pub use sysorder::InitSet;
pub use tbana::TbanaPlugin;

use tbana::TbanaBundle;

use crate::{
    fotocell::{
        on_fotocell_blocked, on_fotocell_unblocked, FotocellAssets, FotocellBundle, FotocellPlugin,
    },
    io::{on_parrent_switch, Dio, DioPin, IOStore, IoDevices, IoPlugin, NodeId},
    shiftreg::ShiftRegPlugin,
    sysorder::SysOrderPlugin,
    tbana::{MovimotDQ, PushTo, SpawnTbana4x2, TBanaAssets, TransportWheelBundle},
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
    let n_banor = 8;
    let io_size = 8 * n_banor;
    let device_address: NodeId = 0.into();
    io.digital_inputs
        .insert(device_address, IOStore::new(io_size));
    io.digital_outputs
        .insert(device_address, IOStore::new(io_size));

    let mut translation = Vec3::default();

    let spaceing = 2.1;
    for i in (1..=n_banor).rev() {
        let inputs = io.digital_inputs.get_mut(&device_address).unwrap();
        let inputs = inputs
            .take(4)
            .map(|pin| Dio {
                address: device_address,
                pin,
            })
            .collect_array()
            .unwrap();
        let outputs = io.digital_outputs.get_mut(&device_address).unwrap();
        let outputs = outputs
            .take(6)
            .map(|pin| Dio {
                address: device_address,
                pin,
            })
            .collect_array()
            .unwrap();

        translation.z = spaceing * (i - 1) as f32;
        cmd.trigger(SpawnTbana4x2::new(
            None,
            format!("stn {i}"),
            inputs,
            outputs,
            Transform::from_translation(translation),
        ));
    }
}

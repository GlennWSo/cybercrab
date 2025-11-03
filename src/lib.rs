use bevy::prelude::*;

pub mod fotocell;
pub mod io;
pub mod physics;
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
    shiftreg::{RegisterPosition, ShiftRegPlugin},
    sysorder::SysOrderPlugin,
    tbana::{Direction, InsertTbana4x2, MovimotDQ, PushTo, TBanaAssets, TransportWheelBundle},
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

fn spawn_some_stuff(mut cmd: Commands, mut io: ResMut<IoDevices>) {
    let n_banor = 30;
    let io_size = 8 * n_banor;
    let node: NodeId = 0.into();
    io.digital_inputs.insert(node, IOStore::new(io_size));
    io.digital_outputs.insert(node, IOStore::new(io_size));

    let mut translation = Vec3::default();

    let spaceing = 2.1;

    let new_enitities: Vec<_> = (0..=n_banor).map(|i| cmd.spawn_empty().id()).collect();

    for (i, entity) in new_enitities.iter().enumerate() {
        let inputs = io.digital_inputs.get_mut(&node).unwrap();
        let inputs = inputs
            .take(4)
            .map(|pin| Dio { node, pin })
            .collect_array()
            .unwrap();
        let outputs = io.digital_outputs.get_mut(&node).unwrap();
        let outputs = outputs
            .take(6)
            .map(|pin| Dio { node, pin })
            .collect_array()
            .unwrap();

        translation.z = spaceing * i as f32;
        let push = new_enitities.get(i + 1).map(|ent| PushTo(*ent));
        let from = if i > 0 {
            new_enitities.get(i - 1).map(|ent| tbana::PullFrom(*ent))
        } else {
            None
        };
        cmd.trigger(InsertTbana4x2::new(
            *entity,
            None,
            format!("stn {i}"),
            inputs,
            outputs,
            Transform::from_translation(translation),
            Direction::Forward,
            RegisterPosition(i as u16),
            push,
            from,
        ));
    }
}

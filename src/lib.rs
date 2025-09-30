use bevy::prelude::*;

pub mod io;
pub mod shiftreg;
mod tbana;
pub use tbana::TbanaPlugin;

use tbana::TbanaBundle;

use crate::{
    io::{AttachedThings, FotoCell, IoSlot, IoThing},
    tbana::PushTo,
};

pub struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TbanaPlugin);
        app.add_systems(Startup, spawn_some_stuff);
    }
}

fn spawn_some_stuff(mut cmd: Commands) {
    let input_device = io::InputDevice::<1>::new("InputNode1", 1);
    cmd.spawn((
        input_device,
        related!(AttachedThings[
            (FotoCell, Name::new("Fotocell1"), IoSlot::new(0, io::DataSlice::Bit(0))),
            (FotoCell, Name::new("Fotocell2"), IoSlot::new(0, io::DataSlice::Bit(1))),
            (FotoCell, Name::new("Fotocell3"), IoSlot::new(0, io::DataSlice::Bit(2))),
        ]),
    ));

    let n = 8;

    let last_bundle = TbanaBundle::new(format!("Stn: {}", n));
    let mut id = cmd.spawn(last_bundle).id();
    for i in (1..n).rev() {
        let bundle = (TbanaBundle::new(format!("Stn: {}", i)), PushTo(id));
        id = cmd.spawn(bundle).id();
    }
}

use avian3d::data_structures::bit_vec::BitVec;
use bevy::{platform::collections::HashMap, prelude::*};
// use bitvec::vec::BitVec;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DeviceNetwork>();
        app.register_type::<ConnectedTo>();
        app.init_resource::<DeviceNetwork>();
    }
}

pub type NetAddress = u32;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DeviceNetwork {
    pub address_map: HashMap<NetAddress, Entity>,
}

#[derive(Resource, Default)]
pub struct IODevices {
    pub input: HashMap<NetAddress, BitVec>,
}

#[derive(Component, Reflect)]
pub struct DIOModule<const S: usize> {
    data: [bool; S],
}

impl<const N: usize> DIOModule<N> {
    fn new() -> Self {
        Self { data: [false; N] }
    }
    pub fn spawn(
        cmd: &mut Commands,
        net: &mut DeviceNetwork,
        name: &'static str,
        address: NetAddress,
    ) -> Entity {
        let new_device = Self::new();
        let id = cmd.spawn((new_device, Name::new(name))).id();
        net.address_map.insert(address, id);
        id
    }
}

#[derive(Component, Reflect)]
pub struct ConnectedTo(pub NetAddress);

#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut)]
pub struct DIOPin(pub u16);

#[derive(Bundle)]
pub struct IoThing {
    pub io_device: ConnectedTo,
    pub pin: DIOPin,
}

#[derive(Component, Default, Reflect, Deref, DerefMut)]
pub struct Switch(pub bool);

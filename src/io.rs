use bevy::prelude::*;

struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

trait GetData {
    fn get_data(&self) -> &[u8];
}

#[derive(Component)]
pub struct InputDevice<const N: usize> {
    pub hostname: &'static str,
    pub address: u32,
    data: [u8; N],
}

impl<const N: usize> InputDevice<N> {
    pub fn new(name: &'static str, address: u32) -> Self {
        Self {
            hostname: name,
            address,
            data: [0; N],
        }
    }
}

impl<const N: usize> GetData for InputDevice<N> {
    fn get_data(&self) -> &[u8] {
        &self.data
    }
}

pub enum DataSlice {
    Byte,
    Bit(u8),
    Slice(u8),
}

#[derive(Component)]
#[relationship(relationship_target=AttachedThings)]
pub struct ConnectedTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ConnectedTo)]
pub struct AttachedThings(Vec<Entity>);

#[derive(Component)]
pub struct IoSlot {
    pub address: u8,
    pub offset: DataSlice,
}

impl IoSlot {
    pub fn new(address: u8, offset: DataSlice) -> Self {
        Self { address, offset }
    }
}

#[derive(Bundle)]
pub struct IoThing {
    pub io_device: ConnectedTo,
    pub slot: IoSlot,
}

#[derive(Component)]
pub struct FotoCell;

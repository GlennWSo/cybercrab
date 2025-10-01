use bevy::{platform::collections::HashMap, prelude::*};

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DeviceNetwork>();
        app.register_type::<ConnectedTo>();
        app.register_type::<AttachedThings>();
        app.init_resource::<DeviceNetwork>();
    }
}

pub type Address = u32;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DeviceNetwork {
    address_map: HashMap<Address, Entity>,
}

pub trait ViewData {
    fn view_all(&self) -> &[u8];
    fn view_input(&self) -> &[u8] {
        self.view_all()
    }
    fn view_bit(&self, address: u8, idx: u8) -> bool {
        let mask: u8 = 1 << idx;
        let hit = mask & self.view_all()[address as usize];
        hit > 0
    }
    fn view_input_bit(&self, address: u8, idx: u8) -> bool {
        let mask: u8 = 1 << idx;
        let hit = mask & self.view_input()[address as usize];
        hit > 0
    }
}

trait GetOutput {}

#[derive(Component, Reflect)]
/// input device with NB bytes of IO inputs
pub struct DigitalInputDevice<const NB: usize> {
    data: [u8; NB],
}

impl<const N: usize> ViewData for DigitalInputDevice<N> {
    fn view_all(&self) -> &[u8] {
        &self.data
    }
}

impl<const N: usize> DigitalInputDevice<N> {
    fn new() -> Self {
        Self { data: [0; N] }
    }
    pub fn spawn(
        cmd: &mut Commands,
        net: &mut DeviceNetwork,
        name: &'static str,
        address: Address,
    ) -> Entity {
        let new_device = Self::new();
        let id = cmd.spawn((new_device, Name::new(name))).id();
        net.address_map.insert(address, id);
        id
    }
}

#[derive(Reflect)]
pub enum DataSlice {
    Byte,
    Bit(u8),
    Slice(u8),
}

#[derive(Component, Reflect)]
#[relationship(relationship_target=AttachedThings)]
pub struct ConnectedTo(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = ConnectedTo)]
pub struct AttachedThings(Vec<Entity>);

#[derive(Component, Reflect)]
pub struct IoSlot {
    pub byte_ptr: u8,
    pub slice: DataSlice,
}

impl IoSlot {
    pub fn new(ptr: u8, offset: DataSlice) -> Self {
        Self {
            byte_ptr: ptr,
            slice: offset,
        }
    }
}

#[derive(Bundle)]
pub struct IoThing {
    pub io_device: ConnectedTo,
    pub slot: IoSlot,
}

#[derive(Component)]
pub struct FotoCell;

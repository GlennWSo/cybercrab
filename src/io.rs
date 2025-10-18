use bevy::{platform::collections::HashMap, prelude::*, render::render_resource::AddressMode};
use bitvec::vec::BitVec;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DeviceAddress>();
        app.init_resource::<IoDevices>();
        app.add_observer(on_digital_input_set);
        app.add_observer(on_switch_set);
    }
}

// pub type NodeAddress = u32;

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct DeviceAddress(pub u32);

impl From<u32> for DeviceAddress {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

// #[derive(Resource, Default, Reflect)]
// #[reflect(Resource)]
// pub struct DeviceNetwork {
//     pub address_map: HashMap<NodeAddress, Entity>,
// }

#[derive(Resource, Default, Debug)]
pub struct IoDevices {
    pub digital_inputs: HashMap<DeviceAddress, BitVec<u32>>,
}

#[derive(Component, Reflect)]
pub struct DIOModule<const S: usize> {
    data: [bool; S],
}

impl<const N: usize> DIOModule<N> {
    fn new() -> Self {
        Self { data: [false; N] }
    }
    // pub fn spawn(
    //     cmd: &mut Commands,
    //     net: &mut DeviceNetwork,
    //     name: &'static str,
    //     address: NodeAddress,
    // ) -> Entity {
    //     let new_device = Self::new();
    //     let id = cmd.spawn((new_device, Name::new(name))).id();
    //     net.address_map.insert(address, id);
    //     id
    // }
}

// #[derive(Component, Reflect, Deref, DerefMut)]
// pub struct ConnectedTo(pub NodeAddress);

#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct DIOPin(pub u16);

impl DIOPin {
    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Component, Default, Reflect)]
pub struct Switch;

#[derive(Event, Clone, Copy)]
pub enum SwitchSet {
    Opened,
    Closed,
}

impl From<bool> for SwitchSet {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Closed,
            false => Self::Opened,
        }
    }
}

impl SwitchSet {
    fn bit(&self) -> bool {
        match self {
            SwitchSet::Opened => false,
            SwitchSet::Closed => true,
        }
    }
}

#[derive(Event)]
pub struct DigitalInputSet {
    pub address: DeviceAddress,
    pub pin: DIOPin,
    pub value: bool,
}

fn on_digital_input_set(
    trigger: Trigger<DigitalInputSet>,
    q: Query<(Entity, &DeviceAddress, &DIOPin), With<Switch>>,
    mut cmd: Commands,
) {
    let target_address = trigger.address;
    let target_pin = trigger.pin;
    let switches = q.iter().filter_map(|(id, address, pin)| -> Option<Entity> {
        if *address == target_address && *pin == target_pin {
            Some(id)
        } else {
            None
        }
    });
    for switch in switches {
        cmd.entity(switch).trigger(SwitchSet::from(trigger.value));
    }
}

pub fn on_switch_set(
    trigger: Trigger<SwitchSet>,
    q: Query<(&DeviceAddress, &DIOPin), With<Switch>>,
    mut io: ResMut<IoDevices>,
) {
    let switch_id = trigger.target();
    let Ok((address, pin)) = q.get(switch_id) else {
        return;
    };
    let Ok(bits) = io
        .digital_inputs
        .get_mut(address)
        .ok_or(format!("no device available at {}", address.0))
    else {
        return;
    };
    bits.set(pin.as_usize(), trigger.bit());
}

/// propagate on_switch
pub fn on_parrent_switch(trigger: Trigger<SwitchSet>, mut cmd: Commands, q: Query<&Children>) {
    let Ok(children) = q.get(trigger.target()) else {
        return;
    };
    for child in children {
        cmd.entity(*child).trigger(*trigger.event());
    }
}

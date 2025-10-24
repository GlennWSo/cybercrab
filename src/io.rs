use bevy::{platform::collections::HashMap, prelude::*};
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

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct DeviceAddress(pub u32);

impl From<u32> for DeviceAddress {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Resource, Default, Debug)]
pub struct IoDevices {
    pub digital_inputs: HashMap<DeviceAddress, BitVec<u32>>,
    pub digital_outputs: HashMap<DeviceAddress, BitVec<u32>>,
}

#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct DIOPin(pub u16);

impl DIOPin {
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Component, Default, Reflect)]
pub struct Switch;

#[derive(EntityEvent, Clone, Copy)]
pub struct SwitchSet {
    pub entity: Entity,
    pub closed: bool,
}

// pub struct DigitalSensor

#[derive(Event)]
pub struct DigitalInputSet {
    pub address: DeviceAddress,
    pub pin: DIOPin,
    pub value: bool,
}

#[derive(Bundle, Reflect)]
pub struct Dio {
    pub address: DeviceAddress,
    pub pin: DIOPin,
}

fn on_digital_input_set(
    trigger: On<DigitalInputSet>,
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
        cmd.trigger(SwitchSet {
            entity: switch,
            closed: true,
        });
    }
}

pub fn on_switch_set(
    trigger: On<SwitchSet>,
    q: Query<(&DeviceAddress, &DIOPin), With<Switch>>,
    mut io: ResMut<IoDevices>,
) {
    let switch_id = trigger.entity;
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
    bits.set(pin.as_usize(), trigger.closed);
}

/// propagate on_switch
pub fn on_parrent_switch(trigger: On<SwitchSet>, mut cmd: Commands, q: Query<&Children>) {
    let Ok(children) = q.get(trigger.entity) else {
        return;
    };
    for child in children {
        cmd.trigger(SwitchSet {
            entity: *child,
            closed: trigger.closed,
        })
    }
}

use bevy::{platform::collections::HashMap, prelude::*};
use bitvec::vec::BitVec;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NodeId>();
        app.init_resource::<IoDevices>();
        app.add_observer(on_digital_input_set);
        app.add_observer(on_switch_set);
    }
}

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy, Deref)]
pub struct NodeId(pub u32);

impl From<u32> for NodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Component)]
pub struct DIOModule {
    pub address: NodeId,
    free_pins: BitVec<u32>,
}

impl DIOModule {
    pub fn new(address: NodeId, n_bits: usize) -> Self {
        let mut free_pins = BitVec::new();
        free_pins.resize(n_bits, true);
        Self { address, free_pins }
    }
    pub fn take_pin(&mut self, pin_idx: usize) -> Option<Dio> {
        let mut is_free = self.free_pins.get_mut(pin_idx)?;
        let not_free = !*is_free;
        if not_free {
            return None;
        }

        *is_free = false;
        Some(Dio {
            address: self.address,
            pin: DioPin(pin_idx as u16),
        })
    }
}

impl Iterator for DIOModule {
    type Item = Dio;

    /// drains the DioModule of free pins
    fn next(&mut self) -> Option<Self::Item> {
        (0..self.free_pins.len()).find_map(|idx| self.take_pin(idx))
    }
}

#[derive(Debug)]
pub struct IOStore {
    pub state: BitVec<u8>,
    taken: BitVec<u8>,
}

impl IOStore {
    pub fn set(&mut self, idx: usize, value: bool) {
        self.state.set(idx, value);
    }
    pub fn get(&self, idx: usize) -> Option<bool> {
        self.state.get(idx).map(|v| *v)
    }
    pub fn new(size: usize) -> Self {
        let mut state = BitVec::new();
        let mut taken = BitVec::new();
        state.resize(size, false);
        taken.resize(size, false);
        Self { state, taken }
    }
    pub fn take_pin(&mut self, idx: usize) -> Option<DioPin> {
        let mut is_taken = self.taken.get_mut(idx)?;
        if *is_taken {
            return None;
        }
        *is_taken = true;
        Some(DioPin(idx as u16))
    }
}

impl Iterator for IOStore {
    type Item = DioPin;

    fn next(&mut self) -> Option<Self::Item> {
        let mut range = 0..self.taken.len();
        range.find_map(|idx| self.take_pin(idx))
    }
}

#[derive(Resource, Default, Debug)]
pub struct IoDevices {
    pub digital_inputs: HashMap<NodeId, IOStore>,
    pub digital_outputs: HashMap<NodeId, IOStore>,
}

#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct DioPin(pub u16);

impl DioPin {
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
    pub address: NodeId,
    pub pin: DioPin,
    pub value: bool,
}

#[derive(Bundle, Reflect, Clone, Copy)]
pub struct Dio {
    pub address: NodeId,
    pub pin: DioPin,
}

fn on_digital_input_set(
    trigger: On<DigitalInputSet>,
    q: Query<(Entity, &NodeId, &DioPin), With<Switch>>,
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
            closed: trigger.value,
        });
    }
}

pub fn on_switch_set(
    trigger: On<SwitchSet>,
    q: Query<(&NodeId, &DioPin), With<Switch>>,
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

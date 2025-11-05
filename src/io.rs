use std::marker::{Send, Sync};

use bevy::{
    ecs::{
        component,
        spawn::{SpawnRelatedBundle, SpawnableList},
    },
    platform::collections::HashMap,
    prelude::*,
};
use bitvec::vec::BitVec;
use itertools::Itertools;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NodeId>();
        app.init_resource::<IoDevices>();
        app.add_observer(on_ui_overide);
        app.add_observer(on_bit_set);
    }
}

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy, Deref)]
#[component(immutable)]
pub struct NodeId(pub u32);

impl From<u32> for NodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Component)]
pub struct IOBits(pub BitVec<u8>);

impl IOBits {
    pub fn new(len: usize) -> Self {
        Self(BitVec::repeat(false, len))
    }
}

#[derive(Component)]
#[relationship(relationship_target=Connections)]
pub struct ConnectedTo(Entity);

#[derive(Component)]
#[relationship_target(relationship=ConnectedTo)]
pub struct Connections(Vec<Entity>);

impl Connections {
    pub fn new(items: Vec<Entity>) -> Self {
        Self(items)
    }
}

#[derive(Component)]
#[component(immutable)]
pub struct Dios {
    pub node: NodeId,
    pub pins: Box<[upin]>,
}
impl Dios {
    fn len(&self) -> usize {
        self.pins.len()
    }
}

impl From<&[Dio]> for Dios {
    fn from(dio_pins: &[Dio]) -> Self {
        let node = dio_pins
            .iter()
            .next()
            .expect("pins should not be empty")
            .node;
        let pins = dio_pins
            .iter()
            .map(|dio| {
                assert!(dio.node == node);
                dio.pin.0
            })
            .collect();
        Self { node, pins }
    }
}

#[derive(Component)]
pub struct Uses(pub Entity);

pub struct Users(Vec<Entity>);

#[derive(Component)]
pub struct Inputs;

#[derive(Component)]
pub struct Outputs;

#[derive(Bundle)]
pub struct InputBundle {
    pub connection: Dios,
    pub bits: IOBits,
    pub marker: Inputs,
}

impl InputBundle {
    pub fn new(dios: Dios) -> Self {
        let bits = IOBits(BitVec::repeat(false, dios.len()));
        Self {
            connection: dios,
            bits,
            marker: Inputs,
        }
    }
}
#[derive(Bundle)]
pub struct OutputBundle {
    pub connection: Dios,
    pub bits: IOBits,
    pub marker: Outputs,
}

impl OutputBundle {
    pub fn new(dios: Dios) -> Self {
        let bits = IOBits(BitVec::repeat(false, dios.len()));
        Self {
            connection: dios,
            bits,
            marker: Outputs,
        }
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
            node: self.address,
            pin: DioPin(pin_idx as u8),
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
        Some(DioPin(idx as u8))
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

impl IoDevices {
    pub fn get_input_bit(&self, node: NodeId, pin: DioPin) -> Option<bool> {
        let device = self.digital_inputs.get(&node)?;
        device.get(pin.as_usize())
    }
    pub fn get_output_bit(&self, node: NodeId, pin: DioPin) -> Option<bool> {
        let device = self.digital_outputs.get(&node)?;
        device.get(pin.as_usize())
    }
    pub fn set_output_bit(&mut self, node: NodeId, pin: DioPin, value: bool) {
        let device = self.digital_outputs.get_mut(&node).unwrap();
        device.set(pin.as_usize(), value);
    }
}

#[allow(non_camel_case_types)]
pub type upin = u8;

#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq, Hash)]
#[component(immutable)]
pub struct DioPin(pub upin);

impl DioPin {
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Component, Default, Reflect)]
pub struct Switch;

#[derive(Component, Default, Reflect)]
pub struct Coil;

#[derive(EntityEvent, Clone, Copy)]
pub struct SwitchSet {
    pub entity: Entity,
    pub closed: bool,
    pub kind: Io,
}

// pub struct DigitalSensor

#[derive(Debug, Clone, Copy)]
pub enum Io {
    Input,
    Output,
}

#[derive(Event)]
pub struct UIOveride {
    pub address: NodeId,
    pub pin: DioPin,
    pub value: bool,
    pub kind: Io,
}

#[derive(Bundle, Reflect, Clone, Copy, Debug)]
pub struct Dio {
    pub node: NodeId,
    pub pin: DioPin,
}

fn on_ui_overide(
    trigger: On<UIOveride>,
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
            kind: trigger.kind,
        });
    }
}

pub fn on_bit_set(
    trigger: On<SwitchSet>,
    q: Query<(&NodeId, &DioPin), With<Switch>>,
    mut io: ResMut<IoDevices>,
) {
    let switch_id = trigger.entity;
    let Ok((address, pin)) = q.get(switch_id) else {
        return;
    };

    let store = match trigger.kind {
        Io::Input => &mut io.digital_inputs,
        Io::Output => &mut io.digital_outputs,
    };

    let Ok(bits) = store
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
            kind: trigger.kind,
        })
    }
}

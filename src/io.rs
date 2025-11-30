use std::{
    marker::{Send, Sync},
    num::{NonZero, NonZeroU8},
    option::Option,
};

use bevy::prelude::*;
use bitvec::{
    ptr::{BitRef, Const},
    vec::BitVec,
};
use itertools::Itertools;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Address>();
    }
}

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy, Deref)]
#[component(immutable)]
pub struct Address(pub u32);

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Component, Deref)]
pub struct Memory(BitVec<u8>);

impl Memory {
    pub fn new_bytes(n_bytes: usize) -> Self {
        Self::new_bits(n_bytes * 8)
    }

    pub fn new_bits(n_bits: usize) -> Self {
        Self(BitVec::repeat(false, n_bits))
    }
    pub fn get(&self, i: usize) -> Option<BitRef<Const, u8>> {
        self.0.get(i)
    }
}

#[derive(Component)]
pub struct InputDevice;

#[derive(Component)]
pub struct OutputDevice;

#[derive(Component, Deref, DerefMut, Reflect, Default)]
pub struct Switch(bool);

#[derive(Component, Deref, DerefMut, Reflect, Default)]
pub struct Coil(bool);

#[derive(Bundle)]
pub struct InputNodeBundle {
    pub memory: Memory,
    pub address: Address,
    marker: InputDevice,
    pub free_pins: FreePins,
}

impl InputNodeBundle {
    pub fn new(address: Address, n_bits: usize) -> Self {
        Self {
            memory: Memory::new_bits(n_bits),
            marker: InputDevice,
            address,
            free_pins: FreePins::new(n_bits),
        }
    }
}

#[derive(Bundle)]
pub struct OutputNodeBundle {
    pub memory: Memory,
    pub address: Address,
    marker: OutputDevice,
    pub free_pins: FreePins,
}

impl OutputNodeBundle {
    pub fn new(address: Address, n_bits: usize) -> Self {
        Self {
            memory: Memory::new_bits(n_bits),
            marker: OutputDevice,
            free_pins: FreePins::new(n_bits),
            address,
        }
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Reflect)]
#[component(immutable)]
pub struct PinIndex(pub u8);

#[derive(Component, Debug)]
pub struct FreePins(BitVec<u8>);

#[derive(Component, Reflect, Clone, Deref, Copy)]
#[component(immutable)]
pub struct StaticPins<const N: usize>([PinIndex; N]);

pub struct PinnedTo<const N: usize> {
    pub to: Entity,
    pub pins: StaticPins<N>,
}

impl FreePins {
    pub fn len(&self) -> usize {
        self.0.iter().filter(|v| **v).count()
    }
}

impl Iterator for FreePins {
    type Item = PinIndex;

    fn next(&mut self) -> Option<Self::Item> {
        for (idx, mut bit) in self.0.iter_mut().enumerate() {
            if *bit {
                *bit = false;
                return Some(PinIndex(idx as u8));
            }
        }
        None
    }
}

impl FreePins {
    pub fn new(size: usize) -> Self {
        Self(BitVec::repeat(true, size))
    }

    pub fn take_pins<const N: usize>(&mut self) -> Option<StaticPins<N>> {
        self.take(N).collect_array().map(|x| StaticPins(x))
    }
}

#[derive(Component, Copy, Clone, Reflect)]
#[relationship(relationship_target=WireConnections)]
pub struct WiredTo(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship=WiredTo)]
pub struct WireConnections(Vec<Entity>);

#[derive(EntityEvent, Clone, Copy)]
pub struct SwitchSet {
    #[event_target]
    pub target: Entity,
    pub slot: PinIndex,
    pub value: bool,
}

pub fn on_switch_set(trigger: On<SwitchSet>, mut io_mem: Query<&mut Memory>) {
    let Ok(mut memory) = io_mem.get_mut(trigger.target) else {
        return;
    };
    memory.0.set(trigger.slot.0 as usize, trigger.value);
}

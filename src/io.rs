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
pub struct InputNode;

#[derive(Component, Deref, DerefMut, Reflect, Default)]
pub struct Switch(bool);

#[derive(Component)]
pub struct Coil;

#[derive(Bundle)]
pub struct InputNodeBundle {
    pub memory: Memory,
    pub address: Address,
    pub slots: FreePins,
    marker: InputNode,
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Reflect)]
pub struct PinIndex(pub u8);

#[derive(Component)]
pub struct FreePins(BitVec<u8>);

#[derive(Component, Default, Reflect)]
pub struct TakenPins(Vec<PinIndex>);

impl TakenPins {
    pub fn iter(&self) -> impl Iterator<Item = &PinIndex> {
        self.0.iter()
    }
}

impl FromIterator<PinIndex> for TakenPins {
    fn from_iter<T: IntoIterator<Item = PinIndex>>(iter: T) -> Self {
        TakenPins(iter.into_iter().collect())
    }
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
        Self(BitVec::repeat(false, size))
    }
    pub fn take_pins(&mut self, n: usize) -> Option<TakenPins> {
        if self.len() < n {
            return None;
        }
        Some(self.take(n).collect())
    }
}

#[derive(Component, Copy, Clone, Reflect)]
#[relationship(relationship_target=InputPins)]
pub struct WiredTo(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship=WiredTo)]
pub struct InputPins(Vec<Entity>);

#[derive(Component, Clone, Copy)]
#[relationship(relationship_target=OutputPins)]
pub struct OutputPinsTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship=OutputPinsTo)]
pub struct OutputPins(Vec<Entity>);

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

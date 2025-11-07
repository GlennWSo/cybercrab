use std::{
    marker::{Send, Sync},
    num::{NonZero, NonZeroU8},
};

use bevy::prelude::*;
use bitvec::vec::BitVec;
use itertools::Itertools;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ip4>();
    }
}

#[derive(Component, Reflect, Default, Hash, PartialEq, Eq, Debug, Clone, Copy, Deref)]
#[component(immutable)]
pub struct Ip4(pub u32);

impl From<u32> for Ip4 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Memory(pub BitVec<u8>);

impl Memory {
    pub fn new_bytes(n_bytes: usize) -> Self {
        Self::new_bits(n_bytes * 8)
    }

    pub fn new_bits(n_bits: usize) -> Self {
        Self(BitVec::repeat(false, n_bits))
    }
}

#[derive(Bundle)]
pub struct DiNode {
    memory: Memory,
    address: Ip4,
}

#[derive(Bundle)]
pub struct DqNode {
    memory: Memory,
    address: Ip4,
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[component(immutable)]
/// Gives an enitity a number of pins
pub struct PinTerminals(pub NonZeroU8);

impl PinTerminals {
    pub fn new(non_zero: u8) -> Self {
        Self(NonZero::new(non_zero).unwrap())
    }
}

impl Default for PinTerminals {
    /// One Pin
    fn default() -> Self {
        Self(NonZeroU8::MIN)
    }
}

#[derive(Component, Copy, Clone, Reflect)]
#[relationship(relationship_target=InputPins)]
pub struct InputPinsTo(pub Entity);

#[derive(Component, Clone, Copy)]
#[relationship_target(relationship=InputPinsTo)]
pub struct InputPins(Vec<Entity>);

#[derive(Component, Clone, Copy)]
#[relationship(relationship_target=OutputPins)]
pub struct OutputPinsTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship=OutputPinsTo)]
pub struct OutputPins(Vec<Entity>);

#[derive(EntityEvent, Clone, Copy)]
pub struct SwitchSet {
    pub entity: Entity,
    pub closed: bool,
}

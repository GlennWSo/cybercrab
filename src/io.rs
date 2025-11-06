use std::marker::{Send, Sync};

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

#[derive(Component)]
#[relationship(relationship_target=InputPins)]
pub struct InputPinTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship=InputPinTo)]
pub struct InputPins(Vec<Entity>);

#[derive(EntityEvent, Clone, Copy)]
pub struct SwitchSet {
    pub entity: Entity,
    pub closed: bool,
}

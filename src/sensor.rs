use bevy::prelude::*;

#[derive(Component)]
pub struct FrontPosition(pub Entity);

#[derive(Component)]
pub struct BackPosition(pub Entity);

#[derive(Component)]
pub struct FrontProximity(pub Entity);

#[derive(Component)]
pub struct BackProximity(pub Entity);

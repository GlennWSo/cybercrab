use bevy::prelude::*;

pub struct SkiftRegPlugin;

impl Plugin for SkiftRegPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Slot>();
    }
}

#[derive(Component)]
pub struct Detail;

#[derive(Component, Deref, Reflect, Default)]
pub struct Slot {
    pub detail: Option<Entity>,
}

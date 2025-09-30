use std::borrow::Cow;

use bevy::prelude::*;

use crate::shiftreg::Slot;

pub struct TbanaPlugin;

impl Plugin for TbanaPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AutoMode>();
        app.register_type::<PushTo>();
        app.register_type::<Reciver>();
        app.register_type::<PullFrom>();
        app.register_type::<Giver>();
    }
}

#[derive(Bundle)]
pub struct TbanaBundle {
    pub tbana: TransportBana,
    pub name: Name,
    pub auto: AutoMode,
    pub slot: Slot,
}

impl TbanaBundle {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let auto = AutoMode::default();
        let slot = Slot::default();
        let name = Name::new(name);
        Self {
            name,
            tbana: TransportBana,
            auto,
            slot,
        }
    }
}

#[derive(Component)]
struct TransportBana;

#[derive(Reflect, Component, Default, Deref)]
struct AutoMode {
    enabled: bool,
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = Reciver )]
/// Pushes production details to other
pub struct PushTo(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PushTo)]
pub struct Reciver(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = Giver )]
pub struct PullFrom(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship=PullFrom)]
pub struct Giver(Vec<Entity>);

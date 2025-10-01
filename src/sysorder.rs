use bevy::prelude::*;

pub struct SysOrderPlugin;

impl Plugin for SysOrderPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Startup, (InitSet::LoadAssets, InitSet::Spawn).chain());
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InitSet {
    LoadAssets,
    Spawn,
}

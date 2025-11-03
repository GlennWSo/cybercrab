use bevy::prelude::*;

use crate::io::SwitchSet;

#[derive(EntityEvent)]
#[entity_event(propagate)]
#[entity_event(auto_propagate)]
pub struct PositionReached {
    pub entity: Entity,
    pub position: SensorPosition,
}

pub fn on_sensor_switch(
    trigger: On<SwitchSet>,
    mut cmd: Commands,
    sensors: Query<&SensorPosition>,
) {
    let entity = trigger.event_target();
    let Ok(&position) = sensors.get(entity) else {
        return;
    };
    cmd.trigger(PositionReached { entity, position })
}

#[derive(Component, Clone, Copy, Reflect)]
pub enum SensorPosition {
    // #[default]
    LimitFront,
    LimitBack,
    LimitLeft,
    LimitRight,
    LimitUp,
    LimitDown,
    ProximityFront,
    ProximityBack,
    ProximityLeft,
    ProximityRight,
    ProximityUp,
    ProximityDown,
}

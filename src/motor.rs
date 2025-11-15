use bevy::prelude::*;

use crate::tbana::StopRunning;

fn on_stop_running_motor(trigger: On<StopRunning>, motors: Query<&Movimot>) {
    todo!()
}

fn motor_effect(
    // target: Query<(&CollidingEntities, &mut LinearVelocity), Without<Movimot>>,
    motors: Query<(&Movimot, &Transform)>,
) {
    todo!()
}

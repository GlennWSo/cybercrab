use avian3d::prelude::*;

#[derive(Default, PhysicsLayer)]
pub enum PhysLayer {
    Detail,
    Component,
    Actuator,
    Sensor,
    #[default]
    Station,
}

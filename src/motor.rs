use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};

use crate::InitSet;

pub struct MotorPlugin;

impl Plugin for MotorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MotorAssets>()
            .register_type::<MovimotBits>()
            .register_type::<Radius>()
            .register_type::<WheelSpeed>()
            .register_type::<MovimotCfg>()
            .add_systems(Startup, setup_motor_assets.in_set(InitSet::LoadAssets));
    }
}
#[derive(Bundle, Default)]
pub struct Movimot {
    io: MovimotBits,
    cfg: MovimotCfg,
}

#[derive(Bundle)]
pub struct WheelBundle {
    wheel: WheelSpeed,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    collider: Collider,
}

impl WheelBundle {
    pub fn new(assets: &MotorAssets) -> Self {
        Self {
            wheel: default(),
            collider: assets.default_wheel_collider.clone(),
            mesh: Mesh3d(assets.default_wheel_mesh.clone()),
            material: MeshMaterial3d(assets.default_wheel_material.clone()),
        }
    }
}

#[derive(EntityEvent)]
pub struct StopRunning(pub Entity);

pub fn on_stop_movimot(trigger: On<StopRunning>, mut motors: Query<&mut MovimotBits>) {
    let target = trigger.event_target();
    let mut motor_bits = motors
        .get_mut(target)
        .expect("observed entity should have a movimot bits");
    *motor_bits = MovimotBits::Stop;
}

#[derive(Component, Reflect)]
#[component(immutable)]
pub struct Radius(f32);

/// surface tangent speed m/s
#[derive(Component, Reflect, Default)]
pub struct WheelSpeed(f32);

#[derive(Component, Reflect, Debug)]
pub struct MovimotCfg {
    /// rotations per second
    pub fast_rps: f32,
    /// rotations per second`
    pub slow_rps: f32,
}

impl Default for MovimotCfg {
    fn default() -> Self {
        Self {
            fast_rps: 10.0,
            slow_rps: 2.0,
        }
    }
}

impl MovimotCfg {
    pub fn rpm(fast_rpm: f32, slow_rpm: f32) -> Self {
        Self {
            fast_rps: fast_rpm / 60.0,
            slow_rps: slow_rpm / 60.0,
        }
    }
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[repr(u8)]
pub enum MovimotBits {
    #[default]
    Stop = 0b000,
    SlowForward = 0b010,
    SlowReverse = 0b100,
    FastForward = 0b011,
    FastReverse = 0b101,
}
#[derive(Resource, Default)]
pub struct MotorAssets {
    pub default_wheel_material: Handle<StandardMaterial>,
    pub default_wheel_mesh: Handle<Mesh>,
    pub default_wheel_collider: Collider,
}

fn setup_motor_assets(
    mut motor_assets: ResMut<MotorAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wheel_radius = 0.05;
    let wheel_width = 0.4;
    let wheel_color = css::AZURE.into();
    motor_assets.default_wheel_mesh = meshes.add(Cylinder::new(wheel_radius, wheel_width));
    motor_assets.default_wheel_material = materials.add(StandardMaterial {
        base_color: wheel_color,
        ..Default::default()
    });
    motor_assets.default_wheel_collider = Collider::cylinder(wheel_radius, wheel_width);
}
